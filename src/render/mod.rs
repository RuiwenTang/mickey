pub(crate) mod command;
pub(crate) mod fragment;
pub(crate) mod raster;

use std::ops::Range;

pub(crate) use command::CommandList;

use crate::gpu::{buffer::StageBuffer, context::PipelineGenerater, pipeline::Pipeline, GPUContext};

use self::command::Command;

pub(crate) trait Renderer {
    fn prepare(&mut self, buffer: &mut StageBuffer, device: &wgpu::Device, queue: &wgpu::Queue);

    fn render<'a>(
        &self,
        buffer: &'a wgpu::Buffer,
        context: &'a GPUContext,
        device: &wgpu::Device,
    ) -> Vec<Command<'a>>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum VertexMode {
    Convex,
    Complex,
}

pub(crate) trait Raster {
    fn rasterize(
        &self,
        buffer: &mut StageBuffer,
    ) -> (
        Range<wgpu::BufferAddress>,
        Range<wgpu::BufferAddress>,
        VertexMode,
        u32,
    );
}

pub(crate) trait Fragment {
    fn get_pipeline_name(&self) -> &'static str;

    fn prepare(&mut self, buffer: &mut StageBuffer, device: &wgpu::Device, queue: &wgpu::Queue);

    fn gen_bind_groups<'a>(
        &self,
        device: &wgpu::Device,
        buffer: &'a wgpu::Buffer,
        pipeline: &'a Pipeline,
    ) -> Vec<wgpu::BindGroup>;
}

pub(crate) struct DummyRenderer<R: Raster, F: Fragment + PipelineGenerater> {
    raster: R,
    fragment: F,
    vertex_range: Range<wgpu::BufferAddress>,
    index_range: Range<wgpu::BufferAddress>,
    vertex_mode: VertexMode,
    draw_count: u32,
}

impl<R: Raster, F: Fragment + PipelineGenerater> DummyRenderer<R, F> {
    pub(crate) fn new(raster: R, fragment: F) -> Self {
        Self {
            raster,
            fragment,
            vertex_range: 0..0,
            index_range: 0..0,
            vertex_mode: VertexMode::Convex,
            draw_count: 0,
        }
    }
}

impl<R: Raster, F: Fragment + PipelineGenerater> Renderer for DummyRenderer<R, F> {
    fn prepare(&mut self, buffer: &mut StageBuffer, device: &wgpu::Device, queue: &wgpu::Queue) {
        let (vertex_range, index_range, vertex_mode, draw_count) = self.raster.rasterize(buffer);
        self.vertex_range = vertex_range;
        self.index_range = index_range;
        self.vertex_mode = vertex_mode;
        self.draw_count = draw_count;

        self.fragment.prepare(buffer, device, queue);
    }

    fn render<'a>(
        &self,
        buffer: &'a wgpu::Buffer,
        context: &'a GPUContext,
        device: &wgpu::Device,
    ) -> Vec<Command<'a>> {
        let pipeline = context.get_pipeline(self.fragment.get_pipeline_name());
        if pipeline.is_none() {
            return vec![];
        }

        let pipeline = pipeline.unwrap();

        let bind_groups = self.fragment.gen_bind_groups(device, buffer, pipeline);

        let state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Never,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Never,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Never,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: Default::default(),
        };

        let raw_pipeline = pipeline.get_pipeline(&state);

        if raw_pipeline.is_none() {
            return vec![];
        }

        let raw_pipeline = raw_pipeline.unwrap();

        return vec![Command::new(
            raw_pipeline,
            buffer.slice(self.vertex_range.clone()),
            buffer.slice(self.index_range.clone()),
            self.draw_count,
            bind_groups,
        )];
    }
}
