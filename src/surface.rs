use nalgebra::Matrix4;

use crate::{
    Color, Content, Draw, EvenOddStencil, MeshType, NonZeroStencil, NoneStencil, Picture,
    PositionChunk, RenderContext, StageBuffer, StrokeStencil,
};

fn mvp_to_buffer(mvp: &Matrix4<f32>) -> [f32; 16] {
    mvp.as_slice()
        .iter()
        .map(|v| *v)
        .collect::<Vec<f32>>()
        .try_into()
        .unwrap()
}

/// Surface wrap a wgpu::Texture as a render target.
/// This is a one-time operation, after flush the surface is no longer usable.
/// No need to present the surface after flush.
pub struct Surface<'a> {
    texture: &'a wgpu::Texture,
    color: Color,
    cmds: Vec<Draw>,
}

impl<'a> Surface<'a> {
    /// Create a new surface from a wgpu::Texture.
    ///
    /// # Arguments
    ///
    /// * `texture` - The wgpu::Texture to wrap as a render target.
    pub fn new(texture: &'a wgpu::Texture) -> Self {
        Surface {
            texture,
            color: Color::transparent(),
            cmds: Vec::new(),
        }
    }

    /// Set the clear color of the surface.
    /// The default clear color is transparent.
    ///
    /// # Arguments
    ///
    /// * `color` - The clear color of the surface.
    pub fn with_clear_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn replay(mut self, picture: &Picture) -> Self {
        self.cmds.extend_from_slice(picture.draws.as_slice());
        self
    }

    /// Flush the content of the surface to the target wgpu::Texture.
    pub fn flush(self, context: &mut RenderContext, device: &wgpu::Device, queue: &wgpu::Queue) {
        let format = self.texture.format();
        let sample_count = 4;
        let vw = self.texture.size().width as f32;
        let vh = self.texture.size().height as f32;
        let mvp = Matrix4::new_orthographic(0.0, vw, vh, 0.0, -1000.0, 1000.0);
        let mut buffer = StageBuffer::new(device);

        let mut commands = Vec::new();
        for cmd in self.cmds {
            let raster = cmd.gen_raster();

            let mesh = raster.do_raster(&cmd.transform(), &mut buffer);

            let transform = cmd.transform();
            let paint = cmd.paint();

            let content = Content::new(
                paint.color,
                format,
                sample_count,
                PositionChunk::new(mvp_to_buffer(&mvp), transform.into(), [0.5, 0.0, 0.0, 0.0]),
            );

            let needs_stencil = match mesh.mesh_type {
                MeshType::Direct => false,
                MeshType::Stroke => false,
                MeshType::EvenOdd => true,
                MeshType::NonZero => true,
            };

            if needs_stencil {
                // draw stencil mask first
                commands.push(content.render_stencil(mesh, &mut buffer, context, device, queue));

                // draw color with stencil mask
                if mesh.mesh_type == MeshType::EvenOdd {
                    commands.push(content.render_color::<EvenOddStencil>(
                        mesh,
                        &mut buffer,
                        context,
                        device,
                        queue,
                    ));
                } else {
                    commands.push(content.render_color::<NonZeroStencil>(
                        mesh,
                        &mut buffer,
                        context,
                        device,
                        queue,
                    ));
                }
            } else {
                if mesh.mesh_type == MeshType::Stroke {
                    commands.push(content.render_color::<StrokeStencil>(
                        mesh,
                        &mut buffer,
                        context,
                        device,
                        queue,
                    ));
                } else {
                    commands.push(content.render_color::<NoneStencil>(
                        mesh,
                        &mut buffer,
                        context,
                        device,
                        queue,
                    ));
                }
            }
        }

        // create a msaa texture and a command encoder
        let msaa = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Attachment"),
            size: self.texture.size(),
            dimension: self.texture.dimension(),
            format: self.texture.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            mip_level_count: 1,
            sample_count,
            view_formats: &[self.texture.format()],
        });

        let depth_stencil = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth stencil"),
            size: self.texture.size(),
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            view_formats: &[wgpu::TextureFormat::Depth24PlusStencil8],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Flush Encoder"),
        });

        // render pass
        {
            let mut desc = wgpu::TextureViewDescriptor {
                label: Some("MSAA AttachmentView"),
                format: Some(self.texture.format()),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                base_array_layer: 0,
                mip_level_count: Some(1),
                array_layer_count: Some(1),
            };

            let msaa_view = msaa.create_view(&desc);
            let ds_view = depth_stencil.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Depth Stencil AttachmentView"),
                format: Some(wgpu::TextureFormat::Depth24PlusStencil8),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                base_array_layer: 0,
                mip_level_count: Some(1),
                array_layer_count: Some(1),
            });

            desc.label = Some("Resolve AttachmentView");

            let resolve_view = self.texture.create_view(&desc);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Flush Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(&resolve_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.color.into()),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &ds_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Discard,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let buffer = buffer.gen_buffer(device, queue);

            if buffer.is_some() {
                let buffer = buffer.unwrap();

                for cmd in commands {
                    cmd.draw(&mut render_pass, &buffer, device);
                }
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}
