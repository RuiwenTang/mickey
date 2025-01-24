use crate::{
    ClipOp, Color, Command, LinearGradient, Matrix3x3, Mesh, PositionChunk, RadialGradient, Rect,
    Renderer, ShapeGeometry, StageBuffer,
};

use super::{
    BWStencilMask, ColorFragment, DepthStencilStateProvider, LinearGradientFragment,
    RadialGradientFragment, RenderContext, ShapeMatrixGeometry, StencilFragment,
    raster::{Raster, RectFillRaster},
};

pub(crate) trait StencilStep {
    fn render_stencil(
        &self,
        mesh: Mesh,
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Command;
}

pub(crate) trait ColorStep {
    fn render_color<S: DepthStencilStateProvider>(
        &self,
        mesh: Mesh,
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Command;
}

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

impl<COLOR> StencilStep for Content<COLOR> {
    fn render_stencil(
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

impl ColorStep for Content<Color> {
    fn render_color<S: DepthStencilStateProvider>(
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

impl ColorStep for Content<(Rect, ClipOp)> {
    fn render_color<S: DepthStencilStateProvider>(
        &self,
        mesh: Mesh,
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Command {
        match self.color.1 {
            ClipOp::Intersect => {
                let rect_raster = RectFillRaster::new(self.color.0);
                let mesh = rect_raster.do_raster(&Matrix3x3::default(), buffer);

                let pos_chunk =
                    PositionChunk::new(self.color.0, Matrix3x3::default(), self.pos_chunk.info());
                let geom = ShapeGeometry::new(pos_chunk);
                let frag = StencilFragment::<S>::new();
                let renderer = Renderer::new(self.format, self.sample_count, geom, frag);

                renderer.render(mesh, buffer, context, device, queue)
            }
            ClipOp::Difference => {
                let geom = ShapeGeometry::new(self.pos_chunk);
                let frag = StencilFragment::<S>::new();
                let renderer = Renderer::new(self.format, self.sample_count, geom, frag);
                renderer.render(mesh, buffer, context, device, queue)
            }
        }
    }
}

impl ColorStep for Content<LinearGradient> {
    fn render_color<S: DepthStencilStateProvider>(
        &self,
        mesh: Mesh,
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Command {
        let geom = ShapeMatrixGeometry::new(self.pos_chunk, self.color.gradient.local_matrix);
        let frag = LinearGradientFragment::<S>::new(self.color.clone());

        let renderer = Renderer::new(self.format, self.sample_count, geom, frag);

        renderer.render(mesh, buffer, context, device, queue)
    }
}

impl ColorStep for Content<RadialGradient> {
    fn render_color<S: DepthStencilStateProvider>(
        &self,
        mesh: Mesh,
        buffer: &mut StageBuffer,
        context: &mut RenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Command {
        let geom = ShapeMatrixGeometry::new(self.pos_chunk, self.color.gradient.local_matrix);
        let frag = RadialGradientFragment::<S>::new(self.color.clone());

        let renderer = Renderer::new(self.format, self.sample_count, geom, frag);
        renderer.render(mesh, buffer, context, device, queue)
    }
}
