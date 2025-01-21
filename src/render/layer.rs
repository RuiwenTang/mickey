use crate::{ClipOp, Color, Draw, Rect};

use super::{
    ColorStep, Command, Content, DifferenceClip, EvenOddStencil, IntersectClip, Mesh, MeshType,
    NonZeroStencil, NoneStencil, PositionChunk, RenderContext, StageBuffer, StencilStep,
    StrokeStencil,
};

fn render_content<C: StencilStep + ColorStep>(
    content: &C,
    mesh: Mesh,
    buffer: &mut StageBuffer,
    context: &mut RenderContext,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pass: &mut Vec<Command>,
) {
    let needs_stencil = match mesh.mesh_type {
        MeshType::Direct => false,
        MeshType::Stroke => false,
        MeshType::EvenOdd => true,
        MeshType::NonZero => true,
    };

    if needs_stencil {
        // draw stencil mask first
        pass.push(content.render_stencil(mesh, buffer, context, device, queue));

        // draw color with stencil mask
        if mesh.mesh_type == MeshType::EvenOdd {
            pass.push(content.render_color::<EvenOddStencil>(mesh, buffer, context, device, queue));
        } else {
            pass.push(content.render_color::<NonZeroStencil>(mesh, buffer, context, device, queue));
        }
    } else {
        if mesh.mesh_type == MeshType::Stroke {
            pass.push(content.render_color::<StrokeStencil>(mesh, buffer, context, device, queue));
        } else {
            pass.push(content.render_color::<NoneStencil>(mesh, buffer, context, device, queue));
        }
    }
}

fn clip_content<C: StencilStep + ColorStep>(
    content: &C,
    op: ClipOp,
    mesh: Mesh,
    buffer: &mut StageBuffer,
    context: &mut RenderContext,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pass: &mut Vec<Command>,
) {
    pass.push(content.render_stencil(mesh, buffer, context, device, queue));

    let even_odd = match mesh.mesh_type {
        MeshType::EvenOdd => true,
        _ => false,
    };

    match op {
        ClipOp::Intersect => {
            if even_odd {
                pass.push(content.render_color::<IntersectClip<EvenOddStencil>>(
                    mesh, buffer, context, device, queue,
                ));
            } else {
                pass.push(content.render_color::<IntersectClip<NonZeroStencil>>(
                    mesh, buffer, context, device, queue,
                ));
            }
        }
        ClipOp::Difference => {
            if even_odd {
                pass.push(content.render_color::<DifferenceClip<EvenOddStencil>>(
                    mesh, buffer, context, device, queue,
                ));
            } else {
                pass.push(content.render_color::<DifferenceClip<NonZeroStencil>>(
                    mesh, buffer, context, device, queue,
                ))
            }
        }
    }
}

pub(crate) struct Layer<'a> {
    view_port: Rect,
    sample_count: u32,
    draws: &'a [Draw],
    target: &'a wgpu::Texture,
    clear_color: Color,
}

impl<'a> Layer<'a> {
    pub(crate) fn new(
        view_port: Rect,
        sample_count: u32,
        draws: &'a [Draw],
        target: &'a wgpu::Texture,
        clear_color: Color,
    ) -> Self {
        Self {
            view_port,
            sample_count,
            draws,
            target,
            clear_color,
        }
    }

    fn target_format(&self) -> wgpu::TextureFormat {
        self.target.format()
    }

    /// Flush the content of the Layer to the target [`wgpu::Texture`].
    pub(crate) fn flush(
        &self,
        context: &mut RenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mut buffer = StageBuffer::new(device);

        let commands = self.render(&mut buffer, context, device, queue);

        let buffer = buffer.gen_buffer(device, queue);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Flush Encoder"),
        });

        {
            let mut render_pass = self.begin_render_pass(&mut encoder, device);

            if buffer.is_some() {
                let buffer = buffer.unwrap();

                for cmd in commands {
                    cmd.draw(&mut render_pass, &buffer, device);
                }
            }
        }

        queue.submit(vec![encoder.finish()]);
    }

    fn render(
        &self,
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Vec<Command> {
        let mut commands = Vec::new();

        let total_depth = (self.draws.len() + 1) as f32;

        for cmd in self.draws {
            let raster = cmd.gen_raster();

            let mesh = raster.do_raster(&cmd.transform(), buffer);

            let paint = cmd.paint();

            let clip_op = cmd.clip_op();

            match clip_op {
                Some(op) => {
                    let content = Content::new(
                        (self.view_port, op),
                        self.target_format(),
                        self.sample_count,
                        PositionChunk::new(self.view_port, cmd.transform(), [
                            cmd.depth() as f32 / total_depth,
                            0.0,
                            0.0,
                            0.0,
                        ]),
                    );

                    clip_content(
                        &content,
                        op,
                        mesh,
                        buffer,
                        context,
                        device,
                        queue,
                        &mut commands,
                    );
                }
                None => {
                    let content = Content::new(
                        paint.color,
                        self.target_format(),
                        self.sample_count,
                        PositionChunk::new(self.view_port, cmd.transform(), [
                            cmd.depth() as f32 / total_depth,
                            0.0,
                            0.0,
                            0.0,
                        ]),
                    );

                    render_content(
                        &content,
                        mesh,
                        buffer,
                        context,
                        device,
                        queue,
                        &mut commands,
                    );
                }
            }
        }

        return commands;
    }

    fn begin_render_pass<'encoder>(
        &self,
        encoder: &'encoder mut wgpu::CommandEncoder,
        device: &wgpu::Device,
    ) -> wgpu::RenderPass<'encoder> {
        if self.sample_count > 1 {
            // create a msaa texture and a command encoder
            let msaa = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("MSAA Attachment"),
                size: self.target.size(),
                dimension: self.target.dimension(),
                format: self.target.format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                mip_level_count: 1,
                sample_count: self.sample_count,
                view_formats: &[self.target.format()],
            });

            let depth_stencil = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("depth stencil"),
                size: self.target.size(),
                mip_level_count: 1,
                sample_count: self.sample_count,
                dimension: wgpu::TextureDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                view_formats: &[wgpu::TextureFormat::Depth24PlusStencil8],
            });
            // render pass
            let mut desc = wgpu::TextureViewDescriptor {
                label: Some("MSAA AttachmentView"),
                format: Some(self.target.format()),
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

            let resolve_view = self.target.create_view(&desc);

            return encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Flush Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(&resolve_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
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
        } else {
            let depth_stencil = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("depth stencil"),
                size: self.target.size(),
                mip_level_count: 1,
                sample_count: self.sample_count,
                dimension: wgpu::TextureDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                view_formats: &[wgpu::TextureFormat::Depth24PlusStencil8],
            });

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

            let resolve_view = self
                .target
                .create_view(&wgpu::TextureViewDescriptor::default());

            return encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Flush Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &resolve_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: wgpu::StoreOp::Store,
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
        }
    }
}
