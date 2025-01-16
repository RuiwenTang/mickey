mod buffer;
mod command;
mod context;
mod fragment;
mod geometry;
mod pipeline;
mod raster;
mod renderer;
mod shader;

pub(crate) use buffer::*;
pub(crate) use command::*;
pub use context::*;
pub(crate) use fragment::*;
pub(crate) use geometry::*;
pub(crate) use pipeline::*;
pub(crate) use raster::*;
pub(crate) use renderer::*;
pub(crate) use shader::*;

/// The raster result. Which indicates if needs to do stencil then fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RasterResult {
    /// The raster result can be rendered directly.
    Direct(StageBufferView, StageBufferView, u32),
    /// The raster result needs to do stencil test then fill. with even-odd winding.
    EvenOdd(StageBufferView, StageBufferView, u32),
    /// The raster result needs to do stencil test then fill. with non-zero winding.
    NonZero(StageBufferView, StageBufferView, u32),
}

impl RasterResult {
    pub(crate) fn raw(&self) -> (StageBufferView, StageBufferView, u32) {
        match self {
            RasterResult::Direct(v, i, c) => (*v, *i, *c),
            RasterResult::EvenOdd(v, i, c) => (*v, *i, *c),
            RasterResult::NonZero(v, i, c) => (*v, *i, *c),
        }
    }
}

/// The renderer responsible for the rendering the geometry shape.
/// First it handled to convert the shape into vertex and index.
/// Then it handled to pick the pipeline for the shape.
/// Finally it generated the Command to do the actual rendering.
pub(crate) trait Renderer {
    fn render(
        &self,
        raster_result: (StageBufferView, StageBufferView, u32),
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        format: wgpu::TextureFormat,
        sample_count: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Command;
}
