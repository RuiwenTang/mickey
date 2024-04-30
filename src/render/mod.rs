pub(crate) mod command;
pub(crate) mod fragment;
pub(crate) mod raster;

use std::ops::Range;

pub(crate) use command::CommandList;

use crate::gpu::{buffer::StageBuffer, pipeline::Pipeline, GPUContext};

use self::{command::Command, fragment::NON_COLOR_PIPELINE_NAME};

pub(crate) trait Renderer {
    fn pipeline_label(&self) -> &'static str;

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
    EvenOddFill,
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

    fn gen_common_bind_groups<'a>(
        &self,
        device: &wgpu::Device,
        buffer: &'a wgpu::Buffer,
        pipeline: &'a Pipeline,
    ) -> wgpu::BindGroup;
}

pub(crate) struct PathRenderer<R: Raster, F: Fragment> {
    format: wgpu::TextureFormat,
    anti_alias: bool,
    raster: R,
    fragment: F,
    vertex_range: Range<wgpu::BufferAddress>,
    index_range: Range<wgpu::BufferAddress>,
    vertex_mode: VertexMode,
    draw_count: u32,
}

impl<R: Raster, F: Fragment> PathRenderer<R, F> {
    pub(crate) fn new(
        format: wgpu::TextureFormat,
        anti_alias: bool,
        raster: R,
        fragment: F,
    ) -> Self {
        Self {
            format,
            anti_alias,
            raster,
            fragment,
            vertex_range: 0..0,
            index_range: 0..0,
            vertex_mode: VertexMode::Convex,
            draw_count: 0,
        }
    }

    fn gen_stencil_command<'a>(
        &self,
        buffer: &'a wgpu::Buffer,
        context: &'a GPUContext,
        device: &wgpu::Device,
    ) -> Command<'a> {
        let pipeline = context
            .get_pipeline(NON_COLOR_PIPELINE_NAME, self.format, self.anti_alias)
            .expect("Can not get non color pipeline");

        let common_group = self
            .fragment
            .gen_common_bind_groups(device, buffer, pipeline);

        let state = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::IncrementWrap,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::DecrementWrap,
                },
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: Default::default(),
        };

        let raw_pipeline = pipeline
            .get_pipeline(&state)
            .expect("Can not get stencil mask pipeline");

        Command::new(
            raw_pipeline,
            buffer.slice(self.vertex_range.clone()),
            buffer.slice(self.index_range.clone()),
            self.draw_count,
            vec![common_group],
        )
    }

    fn gen_stencil_state(&self) -> wgpu::DepthStencilState {
        if self.vertex_mode == VertexMode::Convex {
            wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
                bias: Default::default(),
            }
        } else if self.vertex_mode == VertexMode::EvenOddFill {
            wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::NotEqual,
                        fail_op: wgpu::StencilOperation::Replace,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Replace,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::NotEqual,
                        fail_op: wgpu::StencilOperation::Replace,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Replace,
                    },
                    read_mask: 0x01,
                    write_mask: 0xff,
                },
                bias: Default::default(),
            }
        } else {
            wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::NotEqual,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Replace,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::NotEqual,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Replace,
                    },
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
                bias: Default::default(),
            }
        }
    }
}

impl<R: Raster, F: Fragment> Renderer for PathRenderer<R, F> {
    fn pipeline_label(&self) -> &'static str {
        self.fragment.get_pipeline_name()
    }

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
        let pipeline = context.get_pipeline(
            self.fragment.get_pipeline_name(),
            self.format,
            self.anti_alias,
        );
        if pipeline.is_none() {
            return vec![];
        }

        let mut commands: Vec<Command<'a>> = Vec::new();
        if self.vertex_mode != VertexMode::Convex {
            commands.push(self.gen_stencil_command(buffer, context, device));
        }

        let pipeline = pipeline.unwrap();

        let bind_groups = self.fragment.gen_bind_groups(device, buffer, pipeline);

        let state = self.gen_stencil_state();

        let raw_pipeline = pipeline.get_pipeline(&state);

        if raw_pipeline.is_none() {
            return vec![];
        }

        let raw_pipeline = raw_pipeline.unwrap();

        commands.push(Command::new(
            raw_pipeline,
            buffer.slice(self.vertex_range.clone()),
            buffer.slice(self.index_range.clone()),
            self.draw_count,
            bind_groups,
        ));

        return commands;
    }
}
