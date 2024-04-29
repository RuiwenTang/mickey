use std::{iter, ops::Range};

use smallvec::SmallVec;
use wgpu::util::DeviceExt;

/// CPU buffer to holding vertex, index and uniform data
pub(crate) struct StageBuffer {
    buffer: SmallVec<[u8; 16]>,
    /// minimal alignment of uniform buffer
    alignment: u64,
}

impl StageBuffer {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        Self {
            buffer: SmallVec::with_capacity(128),
            alignment: device.limits().min_uniform_buffer_offset_alignment as u64,
        }
    }

    pub(crate) fn push_data(&mut self, data: &[u8]) -> Range<wgpu::BufferAddress> {
        let start = self.buffer.len() as wgpu::BufferAddress;
        self.buffer.extend_from_slice(data);

        return start..(self.buffer.len() as wgpu::BufferAddress);
    }

    pub(crate) fn push_data_align(&mut self, data: &[u8]) -> Range<wgpu::BufferAddress> {
        let mut start = self.buffer.len() as wgpu::BufferAddress;
        let padding = self.alignment - start % self.alignment;

        if padding != 0 {
            self.buffer.extend(iter::repeat(0).take(padding as usize));
        }

        start = self.buffer.len() as wgpu::BufferAddress;

        self.buffer.extend_from_slice(data);

        return start..(self.buffer.len() as wgpu::BufferAddress);
    }

    pub(crate) fn gen_gpu_buffer(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> wgpu::Buffer {
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

        return buffer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::init_test_context;

    #[test]
    fn test_stage_buffer() {
        let (device, queue) = init_test_context();
        let align = device.limits().min_uniform_buffer_offset_alignment as u64;
        let mut buffer = StageBuffer::new(&device);

        let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let range = buffer.push_data(&data);

        assert_eq!(range.start, 0);
        assert_eq!(range.end, 10);

        let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let range = buffer.push_data_align(&data);

        assert_eq!(range.start, align);
        assert_eq!(range.end, align + 16);

        let g_buffer = buffer.gen_gpu_buffer(&device, &queue);

        assert_eq!(g_buffer.size(), align + 16);
    }
}
