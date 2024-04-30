use std::collections::HashMap;

pub(crate) struct Pipeline {
    pub(crate) groups: Vec<wgpu::BindGroupLayout>,
    pub(crate) _layout: wgpu::PipelineLayout,
    pub(crate) pipelines: HashMap<wgpu::DepthStencilState, wgpu::RenderPipeline>,
}

impl Pipeline {
    pub(crate) fn get_pipeline(
        &self,
        state: &wgpu::DepthStencilState,
    ) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(state)
    }

    pub(crate) fn get_group_layout(&self, slot: usize) -> Option<&wgpu::BindGroupLayout> {
        self.groups.get(slot)
    }
}

pub(crate) struct PipelineBuilder<'a> {
    format: wgpu::TextureFormat,
    sample_count: u32,
    groups: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
    buffers: Vec<wgpu::VertexBufferLayout<'a>>,
    states: Vec<wgpu::DepthStencilState>,
}

impl<'a> PipelineBuilder<'a> {
    pub(crate) fn new() -> Self {
        PipelineBuilder {
            format: wgpu::TextureFormat::Bgra8Unorm,
            sample_count: 1,
            groups: vec![],
            buffers: vec![],
            states: vec![],
        }
    }

    pub(crate) fn with_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.format = format;
        self
    }

    pub(crate) fn with_sample_count(mut self, count: u32) -> Self {
        self.sample_count = count;
        self
    }

    pub(crate) fn add_group(mut self, group: Vec<wgpu::BindGroupLayoutEntry>) -> Self {
        self.groups.push(group);
        self
    }

    pub(crate) fn add_buffer(mut self, buffer: wgpu::VertexBufferLayout<'a>) -> Self {
        self.buffers.push(buffer);
        self
    }

    pub(crate) fn with_states(mut self, state: Vec<wgpu::DepthStencilState>) -> Self {
        self.states = state;
        self
    }

    pub(crate) fn build(&self, shader: &wgpu::ShaderModule, device: &wgpu::Device) -> Pipeline {
        let bind_groups: Vec<wgpu::BindGroupLayout> = self
            .groups
            .iter()
            .map(|g| {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: g.as_slice(),
                })
            })
            .collect();

        let gr: Vec<&wgpu::BindGroupLayout> = bind_groups.iter().map(|g| g).collect();

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: gr.as_slice(),
            push_constant_ranges: &[],
        });

        let pipelins = self
            .states
            .iter()
            .map(|s| {
                (
                    s.clone(),
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&layout),
                        vertex: wgpu::VertexState {
                            module: shader,
                            entry_point: "vs_main",
                            buffers: self.buffers.as_slice(),
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            unclipped_depth: false,
                            conservative: false,
                        },
                        depth_stencil: Some(s.clone()),
                        multisample: wgpu::MultisampleState {
                            count: self.sample_count,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: shader,
                            entry_point: "fs_main",
                            targets: &[Some(wgpu::ColorTargetState {
                                format: self.format,
                                blend: Some(wgpu::BlendState {
                                    color: wgpu::BlendComponent {
                                        src_factor: wgpu::BlendFactor::One,
                                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                        operation: wgpu::BlendOperation::Add,
                                    },
                                    alpha: wgpu::BlendComponent {
                                        src_factor: wgpu::BlendFactor::One,
                                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                        operation: wgpu::BlendOperation::Add,
                                    },
                                }),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview: None,
                    }),
                )
            })
            .collect::<HashMap<_, _>>();

        Pipeline {
            groups: bind_groups,
            _layout: layout,
            pipelines: pipelins,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::init_test_context;

    #[test]
    fn test_pipeline_builder() {
        let (device, _queue) = init_test_context();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("test shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../render/shaders/solid_color.wgsl").into(),
            ),
        });

        let builder = PipelineBuilder::new();

        let pipeline = builder
            .with_format(wgpu::TextureFormat::Bgra8Unorm)
            // buffer layout
            .add_buffer(wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            })
            // group 0
            .add_group(vec![wgpu::BindGroupLayoutEntry {
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
            }])
            // group 1
            .add_group(vec![wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(4 * 4),
                },
                count: None,
            }])
            .with_states(vec![
                // for Convex Polygon no stencil test
                wgpu::DepthStencilState {
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
                },
                // for Stencil and Cover winding fill
                wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Never,
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
                },
                // for Stencil and Cover even-odd fill
                wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Never,
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
                },
            ])
            .build(&shader, &device);

        assert_eq!(pipeline.pipelines.len(), 3);

        assert!(pipeline
            .get_pipeline(&wgpu::DepthStencilState {
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
            })
            .is_some());

        assert!(pipeline
            .get_pipeline(&wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Never,
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
            })
            .is_some());
    }
}
