pub(crate) mod command;
pub(crate) mod fragment;
pub(crate) mod raster;

use std::ops::Range;

pub(crate) use command::CommandList;

use crate::{
    core::{picture::ClipOp, PathFillType, Point},
    gpu::{buffer::StageBuffer, pipeline::Pipeline, GPUContext},
};

use self::{
    command::Command,
    fragment::{
        state_for_clip_difference, state_for_clip_even_odd_difference,
        state_for_clip_even_odd_intersect, state_for_clip_intersect, state_for_complex_even_odd,
        state_for_complex_winding, state_for_convex_polygon, state_for_no_overlap,
        state_for_stencil_mask, ClipMaskFragment, NON_COLOR_PIPELINE_NAME,
    },
    raster::PathFill,
};

pub(crate) trait Renderer {
    fn pipeline_label(&self) -> &'static str;

    fn prepare(
        &mut self,
        total_depth: f32,
        buffer: &mut StageBuffer,
        context: &GPUContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );

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
    NonOverlap,
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

    fn prepare(
        &mut self,
        depth: f32,
        buffer: &mut StageBuffer,
        context: &GPUContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );

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

pub(crate) struct PathRenderer {
    format: wgpu::TextureFormat,
    anti_alias: bool,
    raster: Box<dyn Raster>,
    fragment: Box<dyn Fragment>,
    depth: f32,
    vertex_range: Range<wgpu::BufferAddress>,
    index_range: Range<wgpu::BufferAddress>,
    vertex_mode: VertexMode,
    draw_count: u32,
}

impl PathRenderer {
    pub(crate) fn new(
        format: wgpu::TextureFormat,
        anti_alias: bool,
        raster: Box<dyn Raster>,
        fragment: Box<dyn Fragment>,
        depth: f32,
    ) -> Self {
        Self {
            format,
            anti_alias,
            raster,
            fragment,
            depth,
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

        let state = state_for_stencil_mask();

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
            state_for_convex_polygon()
        } else if self.vertex_mode == VertexMode::EvenOddFill {
            state_for_complex_even_odd()
        } else if self.vertex_mode == VertexMode::Complex {
            state_for_complex_winding()
        } else {
            state_for_no_overlap()
        }
    }
}

impl Renderer for PathRenderer {
    fn pipeline_label(&self) -> &'static str {
        self.fragment.get_pipeline_name()
    }

    fn prepare(
        &mut self,
        total_depth: f32,
        buffer: &mut StageBuffer,
        context: &GPUContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        (
            self.vertex_range,
            self.index_range,
            self.vertex_mode,
            self.draw_count,
        ) = self.raster.rasterize(buffer);

        self.fragment
            .prepare(self.depth / total_depth, buffer, context, device, queue);
    }

    fn render<'a>(
        &self,
        buffer: &'a wgpu::Buffer,
        context: &'a GPUContext,
        device: &wgpu::Device,
    ) -> Vec<Command<'a>> {
        if self.vertex_range.is_empty() || self.index_range.is_empty() {
            return vec![];
        }
        let pipeline = context.get_pipeline(
            self.fragment.get_pipeline_name(),
            self.format,
            self.anti_alias,
        );
        if pipeline.is_none() {
            return vec![];
        }

        let mut commands: Vec<Command<'a>> = Vec::new();
        if self.vertex_mode != VertexMode::Convex && self.vertex_mode != VertexMode::NonOverlap {
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

pub(crate) struct PathCliper {
    format: wgpu::TextureFormat,
    anti_alias: bool,
    pub(crate) raster: PathFill,
    pub(crate) fragment: ClipMaskFragment,
    pub(crate) op: ClipOp,
    pub(crate) depth: f32,

    vertex_range: Range<wgpu::BufferAddress>,
    index_range: Range<wgpu::BufferAddress>,
    vertex_mode: VertexMode,
    draw_count: u32,

    bounds_vertex_range: Range<wgpu::BufferAddress>,
    bounds_index_range: Range<wgpu::BufferAddress>,
}

impl PathCliper {
    pub(crate) fn new(
        format: wgpu::TextureFormat,
        anti_alias: bool,
        raster: PathFill,
        fragment: ClipMaskFragment,
        op: ClipOp,
        depth: f32,
    ) -> Self {
        Self {
            format,
            anti_alias,
            raster,
            fragment,
            op,
            depth,
            vertex_range: 0..0,
            index_range: 0..0,
            vertex_mode: VertexMode::Convex,
            draw_count: 0,
            bounds_vertex_range: 0..0,
            bounds_index_range: 0..0,
        }
    }

    fn raster_bounds(&self) -> ([Point; 4], [u32; 6]) {
        let left = self.fragment.bounds.left;
        let right = self.fragment.bounds.right;
        let top = self.fragment.bounds.top;
        let bottom = self.fragment.bounds.bottom;

        let points = [
            Point::from(left, top),
            Point::from(right, top),
            Point::from(right, bottom),
            Point::from(left, bottom),
        ];

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        (points, indices)
    }
}

impl Renderer for PathCliper {
    fn pipeline_label(&self) -> &'static str {
        NON_COLOR_PIPELINE_NAME
    }

    fn prepare(
        &mut self,
        total_depth: f32,
        buffer: &mut StageBuffer,
        _context: &GPUContext,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.fragment
            .prepare(self.depth / total_depth, self.op, buffer);

        (
            self.vertex_range,
            self.index_range,
            self.vertex_mode,
            self.draw_count,
        ) = self.raster.rasterize(buffer);

        if self.op == ClipOp::Intersect {
            let (points, indices) = self.raster_bounds();

            self.bounds_vertex_range = buffer.push_data(bytemuck::cast_slice(&points));
            self.bounds_index_range = buffer.push_data(bytemuck::cast_slice(&indices));
        }
    }

    fn render<'a>(
        &self,
        buffer: &'a wgpu::Buffer,
        context: &'a GPUContext,
        device: &wgpu::Device,
    ) -> Vec<Command<'a>> {
        if self.vertex_range.is_empty() || self.index_range.is_empty() {
            return vec![];
        }

        let pipeline = context
            .get_pipeline(NON_COLOR_PIPELINE_NAME, self.format, self.anti_alias)
            .expect("Can not get non color pipeline");

        let mut commands: Vec<Command<'a>> = Vec::new();

        // step 1: draw stencil mask
        {
            let common_group = self.fragment.gen_transform_group(device, buffer, pipeline);

            let state = state_for_stencil_mask();

            let raw_pipeline = pipeline
                .get_pipeline(&state)
                .expect("Can not get stencil mask pipeline");

            commands.push(Command::new(
                raw_pipeline,
                buffer.slice(self.vertex_range.clone()),
                buffer.slice(self.index_range.clone()),
                self.draw_count,
                vec![common_group],
            ));
        }

        // step 2: draw clip mask
        {
            let state = if self.op == ClipOp::Intersect {
                if self.raster.path.fill_type == PathFillType::Winding {
                    state_for_clip_intersect()
                } else {
                    state_for_clip_even_odd_intersect()
                }
            } else {
                if self.raster.path.fill_type == PathFillType::Winding {
                    state_for_clip_difference()
                } else {
                    state_for_clip_even_odd_difference()
                }
            };

            let raw_pipeline = pipeline
                .get_pipeline(&state)
                .expect("Can not get clip mask pipeline");

            if self.op == ClipOp::Intersect {
                let identity_group = self.fragment.gen_identity_group(device, buffer, pipeline);

                commands.push(Command::new(
                    raw_pipeline,
                    buffer.slice(self.bounds_vertex_range.clone()),
                    buffer.slice(self.bounds_index_range.clone()),
                    6,
                    vec![identity_group],
                ));
            } else {
                let common_group = self.fragment.gen_transform_group(device, buffer, pipeline);

                commands.push(Command::new(
                    raw_pipeline,
                    buffer.slice(self.vertex_range.clone()),
                    buffer.slice(self.index_range.clone()),
                    self.draw_count,
                    vec![common_group],
                ));
            }
        }
        return commands;
    }
}
