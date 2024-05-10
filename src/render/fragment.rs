use std::ops::Range;

use nalgebra::{Matrix4, Vector4};

use crate::{
    core::{picture::ClipOp, Color, Rect},
    gpu::{
        buffer::StageBuffer,
        context::PipelineGenerater,
        pipeline::{Pipeline, PipelineBuilder},
    },
};

use super::Fragment;

pub(crate) const SOLID_PIPELINE_NAME: &str = "SolidColor";
pub(crate) const NON_COLOR_PIPELINE_NAME: &str = "NonColor";

pub(crate) fn state_for_convex_polygon() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Greater,
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
}

pub(crate) fn state_for_complex_winding() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Greater,
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

pub(crate) fn state_for_complex_even_odd() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Greater,
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
}

pub(crate) fn state_for_no_overlap() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Greater,
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
}

pub(crate) fn state_for_stencil_mask() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Greater,
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
    }
}

pub(crate) fn state_for_clip_intersect() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Greater,
        stencil: wgpu::StencilState {
            front: wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Replace,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Replace,
            },
            back: wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Replace,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Replace,
            },
            read_mask: 0xff,
            write_mask: 0xff,
        },
        bias: Default::default(),
    }
}

pub(crate) fn state_for_clip_even_odd_intersect() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Greater,
        stencil: wgpu::StencilState {
            front: wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Replace,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Replace,
            },
            back: wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Replace,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Replace,
            },
            read_mask: 0x01,
            write_mask: 0xff,
        },
        bias: Default::default(),
    }
}

pub(crate) fn state_for_clip_difference() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Greater,
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
            read_mask: 0xff,
            write_mask: 0xff,
        },
        bias: Default::default(),
    }
}

pub(crate) fn state_for_clip_even_odd_difference() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Greater,
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
}

struct TransformGroup {
    mvp: Matrix4<f32>,
    transform: Matrix4<f32>,
    info: Vector4<f32>,

    buffer_range: Range<wgpu::BufferAddress>,
}

impl TransformGroup {
    fn new(mvp: Matrix4<f32>, transform: Matrix4<f32>, info: Vector4<f32>) -> Self {
        Self {
            mvp,
            transform,
            info,
            buffer_range: 0..0,
        }
    }

    fn prepare(&mut self, depth: f32, buffer: &mut StageBuffer) {
        let mut transform = smallvec::SmallVec::<[f32; 36]>::new();

        self.info[0] = depth;

        transform.extend_from_slice(self.mvp.as_slice());
        transform.extend_from_slice(self.transform.as_slice());
        transform.extend_from_slice(self.info.as_slice());

        self.buffer_range = buffer.push_data_align(bytemuck::cast_slice(transform.as_slice()));
    }

    fn get_buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.buffer_range.clone()
    }
}

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

pub(crate) struct ColorPipelineGenerator {
    color_writable: bool,
    shader: wgpu::ShaderModule,
    states: Vec<wgpu::DepthStencilState>,
    groups: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
}

impl ColorPipelineGenerator {
    pub(crate) fn solid_color_pipeline(device: &wgpu::Device) -> Box<dyn PipelineGenerater> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Solid Color shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/solid_color.wgsl").into()),
        });
        Box::new(ColorPipelineGenerator {
            color_writable: true,
            shader,
            states: vec![
                // for Convex Polygon no stencil test
                state_for_convex_polygon(),
                // for Stencil and Cover winding fill
                state_for_complex_winding(),
                // for Stencil and Cover even-odd fill
                state_for_complex_even_odd(),
                // for stroke no-overlap fill
                state_for_no_overlap(),
            ],
            groups: vec![
                // group 0
                vec![wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            (std::mem::size_of::<nalgebra::Matrix4<f32>>() * 2 + 16)
                                as wgpu::BufferAddress,
                        ),
                    },
                    count: None,
                }],
                // group 1
                vec![wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4 * 4),
                    },
                    count: None,
                }],
            ],
        })
    }

    pub(crate) fn non_color_pipeline(device: &wgpu::Device) -> Box<dyn PipelineGenerater> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Non Color shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/non_color.wgsl").into()),
        });

        Box::new(ColorPipelineGenerator {
            color_writable: false,
            shader,
            states: vec![
                // for Complex Polygon stencil mask
                state_for_stencil_mask(),
                // for intersect clip mask
                state_for_clip_intersect(),
                // for even-odd intersect clip mask
                state_for_clip_even_odd_intersect(),
                // for difference clip mask
                state_for_clip_difference(),
                // for even-odd difference clip mask
                state_for_clip_even_odd_difference(),
            ],
            groups: vec![
                // group 0
                vec![wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            (std::mem::size_of::<nalgebra::Matrix4<f32>>() * 2 + 16)
                                as wgpu::BufferAddress,
                        ),
                    },
                    count: None,
                }],
            ],
        })
    }
}

impl PipelineGenerater for ColorPipelineGenerator {
    fn gen_pipeline(
        &self,
        format: wgpu::TextureFormat,
        sample_count: u32,
        device: &wgpu::Device,
    ) -> Pipeline {
        let mut builder = PipelineBuilder::new();

        for group in &self.groups {
            builder = builder.add_group(group.clone());
        }

        return builder
            .with_format(format)
            .with_sample_count(sample_count)
            .with_color_writable(self.color_writable)
            .add_buffer(wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            })
            .with_states(self.states.clone())
            .build(&self.shader, device);
    }
}
