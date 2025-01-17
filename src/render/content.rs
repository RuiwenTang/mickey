use crate::{Color, Command, Mesh, PositionChunk, Renderer, ShapeGeometry, StageBuffer};

use super::{
    BWStencilMask, ColorFragment, DepthStencilStateProvider, RenderContext, StencilFragment,
};

pub(crate) struct Content<COLOR> {
    pub(crate) color: COLOR,

    format: wgpu::TextureFormat,
    sample_count: u32,

    pos_chunk: PositionChunk,
}

impl<COLOR> Content<COLOR> {
    pub(crate) fn new(
        color: COLOR,
        format: wgpu::TextureFormat,
        sample_count: u32,
        pos_chunk: PositionChunk,
    ) -> Self {
        Content {
            color,
            format,
            sample_count,
            pos_chunk,
        }
    }
}

impl<COLOR> Content<COLOR> {
    pub(crate) fn render_stencil(
        &self,
        mesh: Mesh,
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Command {
        // when using stencil, it must be a ShapeGeometry

        let geom = ShapeGeometry::new(self.pos_chunk);

        let frag = StencilFragment::<BWStencilMask>::new();

        let renderer = Renderer::new(self.format, self.sample_count, geom, frag);

        renderer.render(mesh, buffer, context, device, queue)
    }
}

impl Content<Color> {
    pub(crate) fn render_color<S: DepthStencilStateProvider>(
        &self,
        mesh: Mesh,
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Command {
        let geom = ShapeGeometry::new(self.pos_chunk);
        let frag = ColorFragment::<S>::new(self.color);

        let renderer = Renderer::new(self.format, self.sample_count, geom, frag);

        renderer.render(mesh, buffer, context, device, queue)
    }
}
