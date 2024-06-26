use std::ops::Range;

use nalgebra::{Matrix4, Vector4};

use crate::gpu::{
    buffer::StageBuffer,
    context::PipelineGenerater,
    pipeline::{Pipeline, PipelineBuilder},
};

pub(crate) mod clip_mask;
pub(crate) mod gradient;
pub(crate) mod solid_color;
pub(crate) mod texture;

pub(crate) use clip_mask::ClipMaskFragment;
pub(crate) use gradient::{GradientColorInfo, LinearGradientFragment, RadialGradientFragment};
pub(crate) use solid_color::SolidColorFragment;
pub(crate) use texture::TextureFragment;

pub(crate) const SOLID_PIPELINE_NAME: &str = "SolidColor";
pub(crate) const NON_COLOR_PIPELINE_NAME: &str = "NonColor";
pub(crate) const LINEAR_GRADIENT_PIPELINE_NAME: &str = "LinearGradient";
pub(crate) const RADIAL_GRADIENT_PIPELINE_NAME: &str = "RadialGradient";
pub(crate) const TEXTURE_PIPELINE_NAME: &str = "TextureColor";
pub(crate) const SOLID_TEXT_PIPELINE_NAME: &str = "SolidText";

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

pub(crate) struct ColorPipelineGenerator {
    color_writable: bool,
    shader: wgpu::ShaderModule,
    states: Vec<wgpu::DepthStencilState>,
    groups: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
}

struct TextPipelineGenerator {
    shader: wgpu::ShaderModule,
    states: Vec<wgpu::DepthStencilState>,
    groups: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
}

impl ColorPipelineGenerator {
    pub(crate) fn solid_color_pipeline(device: &wgpu::Device) -> Box<dyn PipelineGenerater> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Solid Color shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/solid_color.wgsl").into()),
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

    pub(crate) fn linear_gradient_pipeline(device: &wgpu::Device) -> Box<dyn PipelineGenerater> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Linear Gradient shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/linear_gradient.wgsl").into(),
            ),
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
                            (std::mem::size_of::<Matrix4<f32>>() * 2 + 16) as wgpu::BufferAddress,
                        ),
                    },
                    count: None,
                }],
                // group 1
                vec![
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<Matrix4<f32>>() as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                GradientColorInfo,
                            >()
                                as wgpu::BufferAddress),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(4 * 4),
                        },
                        count: None,
                    },
                ],
            ],
        })
    }

    pub(crate) fn radial_gradient_pipeline(device: &wgpu::Device) -> Box<dyn PipelineGenerater> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Linear Gradient shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/radial_gradient.wgsl").into(),
            ),
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
                            (std::mem::size_of::<Matrix4<f32>>() * 2 + 16) as wgpu::BufferAddress,
                        ),
                    },
                    count: None,
                }],
                // group 1
                vec![
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<Matrix4<f32>>() as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                GradientColorInfo,
                            >()
                                as wgpu::BufferAddress),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(4 * 4),
                        },
                        count: None,
                    },
                ],
            ],
        })
    }

    pub(crate) fn image_pipeline(device: &wgpu::Device) -> Box<dyn PipelineGenerater> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/image.wgsl").into()),
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
                vec![
                    // binding 0: ImageTransform
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (std::mem::size_of::<nalgebra::Matrix4<f32>>() + 16)
                                    as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                    // binding 1: ImageBounds
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16 as wgpu::BufferAddress),
                        },
                        count: None,
                    },
                    // binding 2: TextureView
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // binding 3: Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            ],
        })
    }

    pub(crate) fn solid_text_pipeline(device: &wgpu::Device) -> Box<dyn PipelineGenerater> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Solid Text shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/solid_text.wgsl").into()),
        });

        Box::new(TextPipelineGenerator {
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
                vec![
                    // binding 0: Color
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                        },
                        count: None,
                    },
                    // binding 1: TextureView
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // binding 2: Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            ],
        })
    }

    pub(crate) fn non_color_pipeline(device: &wgpu::Device) -> Box<dyn PipelineGenerater> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Non Color shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/non_color.wgsl").into()),
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

impl PipelineGenerater for TextPipelineGenerator {
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
            .with_color_writable(true)
            .add_buffer(wgpu::VertexBufferLayout {
                array_stride: 16,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: 8,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                ],
            })
            .with_states(self.states.clone())
            .build(&self.shader, device);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TransformGroup {
    mvp: Matrix4<f32>,
    transform: Matrix4<f32>,
    info: Vector4<f32>,

    buffer_range: Range<wgpu::BufferAddress>,
}

impl TransformGroup {
    pub(crate) fn new(mvp: Matrix4<f32>, transform: Matrix4<f32>, info: Vector4<f32>) -> Self {
        Self {
            mvp,
            transform,
            info,
            buffer_range: 0..0,
        }
    }

    pub(crate) fn prepare(&mut self, depth: f32, buffer: &mut StageBuffer) {
        let mut transform = smallvec::SmallVec::<[f32; 36]>::new();

        self.info[0] = depth;

        transform.extend_from_slice(self.mvp.as_slice());
        transform.extend_from_slice(self.transform.as_slice());
        transform.extend_from_slice(self.info.as_slice());

        self.buffer_range = buffer.push_data_align(bytemuck::cast_slice(transform.as_slice()));
    }

    pub(crate) fn get_buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.buffer_range.clone()
    }
}
