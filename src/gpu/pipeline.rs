use std::collections::HashMap;

pub(crate) struct Pipeline {
    pub(crate) groups: Vec<wgpu::BindGroupLayout>,
    pub(crate) layout: wgpu::PipelineLayout,
    pub(crate) pipelines: HashMap<wgpu::DepthStencilState, wgpu::RenderPipeline>,
}

pub(crate) struct PipelineBuilder<'a> {
    format: wgpu::TextureFormat,
    groups: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
    buffers: Vec<wgpu::VertexBufferLayout<'a>>,
    states: Vec<wgpu::DepthStencilState>,
}

impl<'a> PipelineBuilder<'a> {
    pub(crate) fn new() -> Self {
        PipelineBuilder {
            format: wgpu::TextureFormat::Bgra8Unorm,
            groups: vec![],
            buffers: vec![],
            states: vec![],
        }
    }

    pub(crate) fn with_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.format = format;
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
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: shader,
                            entry_point: "fs_main",
                            targets: &[Some(wgpu::ColorTargetState {
                                format: self.format,
                                blend: None,
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
            layout: layout,
            pipelines: pipelins,
        }
    }
}
