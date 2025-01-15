use std::{num::NonZeroU64, rc::Rc};

use crate::StageBufferView;

/// Represent the uniform data used in the command.
#[derive(Debug, Clone)]
pub(crate) enum Uniform {
    /// The uniform data is a buffer.
    Buffer(StageBufferView, u32),
    /// The uniform data is a texture.
    Texture(Rc<wgpu::TextureView>, u32),
    /// The uniform data is a sampler.
    Sampler(Rc<wgpu::Sampler>, u32),
}

impl Uniform {
    pub(crate) fn to_binding<'a>(&'a self, buffer: &'a wgpu::Buffer) -> wgpu::BindGroupEntry<'a> {
        match self {
            Uniform::Buffer(buffer_view, binding) => wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer,
                    offset: buffer_view.offset as wgpu::BufferAddress,
                    size: NonZeroU64::new(buffer_view.size as u64),
                }),
            },
            Uniform::Texture(texture, binding) => wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu::BindingResource::TextureView(&texture),
            },
            Uniform::Sampler(sampler, binding) => wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        }
    }
}

/// Represent the group of uniform bindings used in the command.
#[derive(Debug, Clone)]
pub(crate) struct Group {
    pub(crate) index: u32,
    pub(crate) bindings: Vec<Uniform>,
}

impl Group {
    pub(crate) fn bind<'a>(
        &self,
        layout: &'a wgpu::BindGroupLayout,
        buffer: &'a wgpu::Buffer,
        pass: &mut wgpu::RenderPass,
        device: &wgpu::Device,
    ) {
        let bindings: Vec<wgpu::BindGroupEntry> = self
            .bindings
            .iter()
            .map(|uniform| uniform.to_binding(buffer))
            .collect();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: bindings.as_slice(),
            label: None,
        });

        pass.set_bind_group(self.index, &bind_group, &[]);
    }
}

pub(crate) struct Comand {
    pub(crate) pipeline: Rc<wgpu::RenderPipeline>,
    pub(crate) vertex_buffer: StageBufferView,
    pub(crate) index_buffer: StageBufferView,
    pub(crate) groups: Vec<Group>,
    pub(crate) draw_count: u32,
}

impl Comand {
    pub(crate) fn draw(
        &self,
        pass: &mut wgpu::RenderPass,
        buffer: &wgpu::Buffer,
        device: &wgpu::Device,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, buffer.slice(self.vertex_buffer));
        pass.set_index_buffer(buffer.slice(self.index_buffer), wgpu::IndexFormat::Uint32);

        for group in self.groups.iter() {
            group.bind(
                &self.pipeline.get_bind_group_layout(group.index),
                buffer,
                pass,
                device,
            );
        }

        pass.draw_indexed(0..self.draw_count, 0, 0..1);
    }
}
