use wgpu::util::DeviceExt;

/// The stage buffer. Which used to store the data of the vertex buffer and index buffer and uniform buffer.
/// The stage buffer is a temporary buffer used to store the data before copy to the GPU buffer.
pub(crate) struct StageBuffer {
    /// The buffer data.
    buffer: Vec<u8>,
    /// The minimum alignment of the uniform data required by the device.
    alignment: usize,
}

pub(crate) struct StageBufferView {
    offset: usize,
    size: usize,
}

impl StageBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        StageBuffer {
            buffer: Vec::new(),
            alignment: device.limits().min_uniform_buffer_offset_alignment as usize,
        }
    }

    pub fn push<T: bytemuck::Pod>(&mut self, data: &[T]) -> StageBufferView {
        let size = std::mem::size_of::<T>() * data.len();
        let offset = self.buffer.len();

        self.buffer.extend_from_slice(bytemuck::cast_slice(data));

        StageBufferView { offset, size }
    }

    pub fn push_align<T: bytemuck::Pod>(&mut self, data: &[T]) -> StageBufferView {
        let size = std::mem::size_of::<T>() * data.len();
        let mut offset = self.buffer.len();

        let padding = self.alignment - offset % self.alignment;

        if padding > 0 && offset != 0 {
            self.buffer.resize(offset + padding, 0);
            offset += padding;
        }

        self.buffer.extend_from_slice(bytemuck::cast_slice(data));

        StageBufferView { offset, size }
    }

    pub fn gen_buffer(self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<wgpu::Buffer> {
        if self.buffer.len() == 0 {
            return None;
        }

        let total_size = self.buffer.len() as wgpu::BufferAddress;

        let stage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("stage buffer"),
            contents: bytemuck::cast_slice(&self.buffer),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Render Buffer"),
            size: total_size,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("copy buffer"),
        });

        encoder.copy_buffer_to_buffer(&stage_buffer, 0, &buffer, 0, total_size);

        queue.submit(vec![encoder.finish()]);

        return Some(buffer);
    }
}

#[cfg(test)]
mod tests {
    use crate::render::StageBuffer;
    #[test]
    fn test_stage_buffer() {
        let mut buffer = StageBuffer {
            buffer: Vec::new(),
            alignment: 256,
        };
        let view = buffer.push(&[1, 2, 3, 4]);
        assert_eq!(view.offset, 0);
        assert_eq!(view.size, 16);

        let view = buffer.push_align(&[1, 2, 3, 4]);
        assert_eq!(view.offset, 256);
        assert_eq!(view.size, 16);

        let view = buffer.push_align(&[1, 2, 3, 4]);

        assert_eq!(view.offset, 256 * 2);
        assert_eq!(view.size, 16);
    }
}
