use nalgebra::{Matrix4, Vector4};

use crate::{
    core::{ClipOp, Rect},
    gpu::{buffer::StageBuffer, pipeline::Pipeline},
};

use super::TransformGroup;

pub(crate) struct ClipMaskFragment {
    transform: TransformGroup,
    identity: TransformGroup,
    pub(crate) bounds: Rect,
}

impl ClipMaskFragment {
    pub(crate) fn new(vw: f32, vh: f32, transform: Matrix4<f32>) -> Self {
        Self {
            transform: TransformGroup::new(
                Matrix4::new_orthographic(0.0, vw, vh, 0.0, -1000.0, 1000.0),
                transform,
                Vector4::new(0.0, 0.0, 0.0, 0.0),
            ),
            identity: TransformGroup::new(
                Matrix4::new_orthographic(0.0, vw, vh, 0.0, -1000.0, 1000.0),
                Matrix4::identity(),
                Vector4::new(0.0, 0.0, 0.0, 0.0),
            ),
            bounds: Rect::from_xywh(0.0, 0.0, vw, vh),
        }
    }

    pub(crate) fn prepare(&mut self, depth: f32, op: ClipOp, buffer: &mut StageBuffer) {
        self.transform.prepare(depth, buffer);

        if op == ClipOp::Intersect {
            self.identity.prepare(depth, buffer);
        }
    }

    pub(crate) fn gen_transform_group<'a>(
        &self,
        device: &wgpu::Device,
        buffer: &'a wgpu::Buffer,
        pipeline: &'a Pipeline,
    ) -> wgpu::BindGroup {
        let group0_layout = pipeline
            .get_group_layout(0)
            .expect("Pipeline does not contains common transform slot");

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Clip Mask Transform Group"),
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

    pub(crate) fn gen_identity_group<'a>(
        &self,
        device: &wgpu::Device,
        buffer: &'a wgpu::Buffer,
        pipeline: &'a Pipeline,
    ) -> wgpu::BindGroup {
        let group0_layout = pipeline
            .get_group_layout(0)
            .expect("Pipeline does not contains common transform slot");

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Clip Mask Identity Group"),
            layout: &group0_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer,
                    offset: self.identity.get_buffer_range().start,
                    size: wgpu::BufferSize::new(
                        self.identity.get_buffer_range().end
                            - self.identity.get_buffer_range().start,
                    ),
                }),
            }],
        })
    }
}
