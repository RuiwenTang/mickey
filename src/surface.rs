/// Surface wrap a wgpu::Texture as a render target.
/// This is a one-time operation, after flush the surface is no longer usable.
/// No need to present the surface after flush.
pub struct Surface<'a> {
    texture: &'a wgpu::Texture,
}

impl<'a> Surface<'a> {
    /// Create a new surface from a wgpu::Texture.
    ///
    /// # Arguments
    ///
    /// * `texture` - The wgpu::Texture to wrap as a render target.
    pub fn new(texture: &'a wgpu::Texture) -> Self {
        Surface { texture }
    }

    /// Flush the content of the surface to the target wgpu::Texture.
    pub fn flush(self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // create a msaa texture and a command encoder
        let msaa = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Attachment"),
            size: self.texture.size(),
            dimension: self.texture.dimension(),
            format: self.texture.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            mip_level_count: 1,
            sample_count: 4,
            view_formats: &[self.texture.format()],
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

            desc.label = Some("Resolve AttachmentView");

            let resolve_view = self.texture.create_view(&desc);

            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Flush Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(&resolve_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        queue.submit(Some(encoder.finish()));
    }
}
