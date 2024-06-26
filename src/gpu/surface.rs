use crate::{
    core::Picture,
    gpu::{buffer::StageBuffer, GPUContext},
    render::{fragment::NON_COLOR_PIPELINE_NAME, CommandList, Renderer},
};

/// A surface is a wrap around a wgpu::Texture. which can be used to render contents.
pub struct GPUSurface<'a> {
    target: &'a wgpu::Texture,
    anti_alias: bool,
    depth_stencil: wgpu::Texture,
    msaa_texture: Option<wgpu::Texture>,
    logical_width: f32,
    logical_height: f32,

    renders: Vec<Box<dyn Renderer>>,
}

impl<'a> GPUSurface<'a> {
    /// Wrap a wgpu::Texture to a surface.
    ///
    /// # Arguments
    ///
    /// * `target` - The wgpu::Texture to be wrapped.
    /// * `logical_width` - The width of the surface in logical it can be different from actually texture size.
    /// * `logical_height` - The height of the surface in logical it can be different from actually texture size.
    /// * `anti_alias` - Whether to use anti-alias we provide msaa with sample count 4.
    /// * `device` - The wgpu::Device used to create other GPU resources.
    pub fn new(
        target: &'a wgpu::Texture,
        logical_width: f32,
        logical_height: f32,
        anti_alias: bool,
        device: &wgpu::Device,
    ) -> Self {
        let width = target.width();
        let height = target.height();

        let depth_stencil = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth stencil"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: if anti_alias { 4 } else { 1 },
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            view_formats: &[wgpu::TextureFormat::Depth24PlusStencil8],
        });

        let msaa_texture = if anti_alias {
            Some(device.create_texture(&wgpu::TextureDescriptor {
                label: Some("msaa"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: target.format(),
                view_formats: &[target.format()],
            }))
        } else {
            None
        };

        GPUSurface {
            target,
            anti_alias,
            depth_stencil,
            msaa_texture,
            logical_width,
            logical_height,
            renders: Vec::new(),
        }
    }

    /// Replay a picture's draw commands to the surface.
    pub fn replay(&mut self, picture: &Picture) {
        let depth_offset = self.renders.len() as u32;

        for draw in &picture.draws {
            self.renders.push(draw.gen_render(
                self.logical_width,
                self.logical_height,
                self.target.format(),
                self.anti_alias,
                depth_offset,
            ));
        }
    }

    /// Flush the surface to the target texture.
    ///
    /// # Arguments
    ///
    /// * `context` - The GPU context used to create pipelines and other resources.
    /// * `device` - The wgpu::Device used to create other GPU resources.
    /// * `queue` - The wgpu::Queue used to submit commands.
    /// * `clear_color` - The color to clear the target texture. If pass `None`, the target texture will load into inner RenderPass.
    pub fn flush(
        &mut self,
        context: &'a mut GPUContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        clear_color: Option<wgpu::Color>,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("flush"),
        });

        let (target_view, depth_stencil_view, msaa_view) = self.get_views();

        let mut stage_buffer = StageBuffer::new(device);

        // load non color pipeline before visit all renders.
        context.load_pipeline(
            NON_COLOR_PIPELINE_NAME,
            self.target.format(),
            self.anti_alias,
            device,
        );

        let total_depth = (self.renders.len() + 1) as f32;

        for render in &mut self.renders {
            context.load_pipeline(
                render.as_ref().pipeline_label(),
                self.target.format(),
                self.anti_alias,
                device,
            );

            render
                .as_mut()
                .prepare(total_depth, &mut stage_buffer, &context, device, queue);
        }

        let gpu_buffer = stage_buffer.gen_gpu_buffer(device, queue);

        let mut command_list = CommandList::new();
        for render in &mut self.renders {
            let commands = render.as_mut().render(&gpu_buffer, context, device);
            command_list.add_command_list(commands);
        }

        {
            let mut pass = self.begin_render_pass(
                &target_view,
                &depth_stencil_view,
                &msaa_view.as_ref(),
                &mut encoder,
                clear_color,
            );

            pass.set_stencil_reference(0);

            command_list.run(&mut pass);
        }

        queue.submit([encoder.finish()]);
    }

    fn get_views(
        &self,
    ) -> (
        wgpu::TextureView,
        wgpu::TextureView,
        Option<wgpu::TextureView>,
    ) {
        let target_view = self
            .target
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_stencil_view = self
            .depth_stencil
            .create_view(&wgpu::TextureViewDescriptor::default());

        let msaa_view = match self.msaa_texture.as_ref() {
            Some(msaa_texture) => {
                Some(msaa_texture.create_view(&wgpu::TextureViewDescriptor::default()))
            }
            None => None,
        };

        return (target_view, depth_stencil_view, msaa_view);
    }

    fn begin_render_pass(
        &self,
        target: &'a wgpu::TextureView,
        depth_stencil: &'a wgpu::TextureView,
        msaa: &Option<&'a wgpu::TextureView>,
        encoder: &'a mut wgpu::CommandEncoder,
        clear_color: Option<wgpu::Color>,
    ) -> wgpu::RenderPass<'a> {
        if self.anti_alias {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("OnScreen render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: msaa.unwrap(),
                    resolve_target: Some(&target),
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some(clear_color) => wgpu::LoadOp::Clear(clear_color),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_stencil,
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
            })
        } else {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("OnScreen render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some(clear_color) => wgpu::LoadOp::Clear(clear_color),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_stencil,
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
            })
        }
    }
}
