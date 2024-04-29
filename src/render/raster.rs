use super::Raster;

pub(crate) struct DummyRaster {
    points: [f32; 6],
}

impl DummyRaster {
    pub(crate) fn new(points: [f32; 6]) -> Self {
        Self { points }
    }
}

impl Raster for DummyRaster {
    fn rasterize(
        &self,
        buffer: &mut crate::gpu::buffer::StageBuffer,
    ) -> (
        std::ops::Range<wgpu::BufferAddress>,
        std::ops::Range<wgpu::BufferAddress>,
        super::VertexMode,
        u32,
    ) {
        let indices: [u32; 3] = [0, 1, 2];

        let vertex_range = buffer.push_data(bytemuck::cast_slice(&self.points));
        let index_range = buffer.push_data(bytemuck::cast_slice(indices.as_slice()));

        (vertex_range, index_range, super::VertexMode::Convex, 3)
    }
}
