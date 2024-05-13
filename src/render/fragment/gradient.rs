use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Vector4};

use crate::{
    core::{Color, LinearGradient, TileMode},
    gpu::{buffer::StageBuffer, pipeline::Pipeline},
    render::Fragment,
};

use super::{TransformGroup, LINEAR_GRADIENT_PIPELINE_NAME};

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(crate) struct GradientColorInfo {
    counts: [u32; 4],
    colors: [Color; 16],
    stops: [f32; 16],
}

impl GradientColorInfo {
    fn new(colors: &Vec<Color>, stops: Option<&Vec<f32>>, tile_mode: TileMode) -> Self {
        let mut color_arr: [Color; 16] = [Color::transparent(); 16];
        let mut stop_arr: [f32; 16] = [0.0; 16];
        let mut count = 0;
        let mut stop_count = 0;
        for (i, color) in colors.iter().enumerate() {
            color_arr[i] = *color;
            count += 1;
        }

        if let Some(stops) = stops {
            if stops.len() == count {
                stop_count = count;
                for (i, stop) in stops.iter().enumerate() {
                    stop_arr[i] = *stop;
                }
            }
        }

        Self {
            counts: [count as u32, stop_count as u32, tile_mode as u32, 0],
            colors: color_arr,
            stops: stop_arr,
        }
    }
}

pub(crate) struct LinearGradientFragment {
    gradient_info: GradientColorInfo,
    transform: TransformGroup,
    matrix: Matrix4<f32>,
    pts: [f32; 4],

    // ranges
    gradient_info_range: Range<wgpu::BufferAddress>,
    matrix_range: Range<wgpu::BufferAddress>,
    pts_range: Range<wgpu::BufferAddress>,
}

impl LinearGradientFragment {
    pub(crate) fn new(
        gradient: &LinearGradient,
        vw: f32,
        vh: f32,
        transform: Matrix4<f32>,
    ) -> Self {
        let gradient_info = GradientColorInfo::new(
            &gradient.colors,
            if gradient.stops.is_empty() {
                None
            } else {
                Some(&gradient.stops)
            },
            gradient.tile_mode,
        );

        let matrix = if gradient.matrix.is_identity(f32::EPSILON) {
            gradient.matrix.clone()
        } else {
            if gradient.matrix.is_invertible() {
                gradient.matrix.try_inverse().unwrap()
            } else {
                Matrix4::identity()
            }
        };

        Self {
            gradient_info,
            transform: TransformGroup::new(
                Matrix4::new_orthographic(0.0, vw, vh, 0.0, -1000.0, 1000.0),
                transform,
                Vector4::new(0.0, 0.0, 0.0, 0.0),
            ),
            matrix,
            pts: [gradient.p1.x, gradient.p1.y, gradient.p2.x, gradient.p2.y],
            gradient_info_range: 0..0,
            matrix_range: 0..0,
            pts_range: 0..0,
        }
    }
}

impl Fragment for LinearGradientFragment {
    fn get_pipeline_name(&self) -> &'static str {
        LINEAR_GRADIENT_PIPELINE_NAME
    }

    fn prepare(
        &mut self,
        depth: f32,
        buffer: &mut StageBuffer,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.transform.prepare(depth, buffer);

        self.gradient_info_range =
            buffer.push_data_align(bytemuck::cast_slice(&[self.gradient_info]));

        self.matrix_range = buffer.push_data_align(bytemuck::cast_slice(self.matrix.as_slice()));

        self.pts_range = buffer.push_data_align(bytemuck::cast_slice(&self.pts));
    }

    fn gen_bind_groups<'a>(
        &self,
        device: &wgpu::Device,
        buffer: &'a wgpu::Buffer,
        pipeline: &'a Pipeline,
    ) -> Vec<wgpu::BindGroup> {
        // group 1 color uniform
        let group1_layout = pipeline.get_group_layout(1);

        if group1_layout.is_none() {
            return vec![];
        }

        let group1_layout = group1_layout.unwrap();

        vec![
            // goup 0
            self.gen_common_bind_groups(device, buffer, pipeline),
            // group 1
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Linear Gradient Group"),
                layout: &group1_layout,
                entries: &[
                    // binding 0: gradient matrix
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: buffer,
                            offset: self.matrix_range.start,
                            size: wgpu::BufferSize::new(
                                self.matrix_range.end - self.matrix_range.start,
                            ),
                        }),
                    },
                    // binding 1: color info
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: buffer,
                            offset: self.gradient_info_range.start,
                            size: wgpu::BufferSize::new(
                                self.gradient_info_range.end - self.gradient_info_range.start,
                            ),
                        }),
                    },
                    // binding 2: pts info
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: buffer,
                            offset: self.pts_range.start,
                            size: wgpu::BufferSize::new(self.pts_range.end - self.pts_range.start),
                        }),
                    },
                ],
            }),
        ]
    }

    fn gen_common_bind_groups<'a>(
        &self,
        device: &wgpu::Device,
        buffer: &'a wgpu::Buffer,
        pipeline: &'a Pipeline,
    ) -> wgpu::BindGroup {
        let group0_layout = pipeline
            .get_group_layout(0)
            .expect("common group at slot 0 can not be get!");

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Common Transform Group"),
            layout: &group0_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer,
                    offset: self.transform.get_buffer_range().start,
                    size: wgpu::BufferSize::new(
                        self.transform.get_buffer_range().end
                            - self.transform.get_buffer_range().start,
                    ),
                }),
            }],
        })
    }
}
