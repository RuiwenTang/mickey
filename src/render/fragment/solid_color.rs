use std::ops::Range;

use nalgebra::{Matrix4, Vector4};

use crate::{
    core::Color,
    gpu::{buffer::StageBuffer, pipeline::Pipeline},
    render::Fragment,
};

use super::{TransformGroup, SOLID_PIPELINE_NAME};

pub(crate) struct SolidColorFragment {
    color: Color,
    transform: TransformGroup,
    color_range: Range<wgpu::BufferAddress>,
}

impl SolidColorFragment {
    pub(crate) fn new(color: Color, vw: f32, vh: f32, transform: Matrix4<f32>) -> Self {
        Self {
            color,
            transform: TransformGroup::new(
                Matrix4::new_orthographic(0.0, vw, vh, 0.0, -1000.0, 1000.0),
                transform,
                Vector4::new(0.0, 0.0, 0.0, 0.0),
            ),
            color_range: 0..0,
        }
    }
}

impl Fragment for SolidColorFragment {
    fn get_pipeline_name(&self) -> &'static str {
        SOLID_PIPELINE_NAME
    }

    fn prepare(
        &mut self,
        depth: f32,
        buffer: &mut StageBuffer,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.transform.prepare(depth, buffer);

        self.color_range = buffer.push_data_align(bytemuck::cast_slice(&[self.color]));
    }

    fn gen_bind_groups<'a>(
        &self,
        device: &wgpu::Device,
        buffer: &'a wgpu::Buffer,
        pipeline: &'a Pipeline,
    ) -> Vec<wgpu::BindGroup> {
        // group 0 common transform
        let group0_layout = pipeline.get_group_layout(0);
        // group 1 color uniform
        let group1_layout = pipeline.get_group_layout(1);

        if group0_layout.is_none() || group1_layout.is_none() {
            return vec![];
        }

        let group0_layout = group0_layout.unwrap();
        let group1_layout = group1_layout.unwrap();

        vec![
            // goup 0
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("solid_color_transform"),
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
            }),
            // group 1
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("solid_color_color"),
                layout: &group1_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: buffer,
                        offset: self.color_range.start,
                        size: wgpu::BufferSize::new(self.color_range.end - self.color_range.start),
                    }),
                }],
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
            label: Some("NonColor Common Group"),
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
