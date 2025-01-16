use std::{collections::HashMap, ops::DerefMut, rc::Rc};

/// The pipeline key. Which used to identify the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PipelineKey {
    pub(crate) label: String,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) sample_count: u32,
    pub(crate) ds: wgpu::DepthStencilState,
}

impl PipelineKey {
    pub(crate) fn create_pipeline(
        &self,
        label: &String,
        device: &wgpu::Device,
        vs: &wgpu::ShaderModule,
        fs: &wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout,
        attribute: &[wgpu::VertexAttribute],
        attr_stride: wgpu::BufferAddress,
    ) -> Rc<wgpu::RenderPipeline> {
        let pipeline_label = format!("{}-{}", label, self.label);

        let attachments = [Some(wgpu::ColorTargetState {
            format: self.format,
            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let desc = wgpu::RenderPipelineDescriptor {
            label: Some(&pipeline_label),
            layout: Some(layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: vs,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: attr_stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: attribute,
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: fs,
                entry_point: Some("fs_main"),
                targets: &attachments,
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(self.ds.clone()),
            multisample: wgpu::MultisampleState {
                count: self.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        };

        let pipeline = Rc::new(device.create_render_pipeline(&desc));

        return pipeline;
    }
}

pub(crate) struct Pipeline {
    pub(crate) label: String,
    pub(crate) vs: Rc<wgpu::ShaderModule>,
    pub(crate) fs: Rc<wgpu::ShaderModule>,
    pub(crate) layout: wgpu::PipelineLayout,
    pub(crate) attribute: Vec<wgpu::VertexAttribute>,
    pub(crate) attr_stride: wgpu::BufferAddress,

    pub(crate) pipelines: HashMap<PipelineKey, Rc<wgpu::RenderPipeline>>,
}

impl Pipeline {
    pub(crate) fn new(
        label: String,
        vs: Rc<wgpu::ShaderModule>,
        fs: Rc<wgpu::ShaderModule>,
        layout: wgpu::PipelineLayout,
        attribute: Vec<wgpu::VertexAttribute>,
        attr_stride: wgpu::BufferAddress,
    ) -> Self {
        Pipeline {
            label,
            vs,
            fs,
            layout,
            attribute,
            attr_stride,
            pipelines: HashMap::new(),
        }
    }

    pub(crate) fn get_varient(
        &mut self,
        key: &PipelineKey,
        device: &wgpu::Device,
    ) -> Option<Rc<wgpu::RenderPipeline>> {
        if self.pipelines.contains_key(key) {
            return self.pipelines.get(key).cloned();
        }

        // create pipeline based on the key
        let pipeline = key.create_pipeline(
            &self.label,
            device,
            &self.vs,
            &self.fs,
            &self.layout,
            &self.attribute,
            self.attr_stride,
        );

        self.pipelines.insert(key.clone(), pipeline.clone());

        return Some(pipeline);
    }
}
