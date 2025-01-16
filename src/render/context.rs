use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::Pipeline;

/// The shader generator. Which used to generate the shader code dynamically.
/// The shader module will be cached by the context.
/// The shader name is used to identify the shader module. Implementation should ensure the name is unique.
pub(crate) trait ShaderGenerator {
    /// The shader name which used to identify the shader module.
    fn shader_name(&self) -> String;

    /// The shader code.
    fn shader_code(&self) -> String;
}

pub(crate) trait LayoutGenerator {
    fn generate(&self, device: &wgpu::Device) -> wgpu::PipelineLayout;
}

/// The render context. Which used to cache the pipeline and other resources.
/// Created by the rendering engine. And shared the context to all the surfaces.
/// Can help reduce the resource usage.
pub struct RenderContext {
    pub(crate) shaders: HashMap<String, Rc<wgpu::ShaderModule>>,
    pub(crate) pipelines: HashMap<String, Rc<RefCell<Pipeline>>>,
}

pub(crate) struct PipelineQuery<'a> {
    pub(crate) label: String,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) sample_count: u32,
    pub(crate) ds: wgpu::DepthStencilState,
    pub(crate) attributes: Vec<wgpu::VertexAttribute>,
    pub(crate) attr_stride: wgpu::BufferAddress,

    pub(crate) vs_generator: &'a dyn ShaderGenerator,
    pub(crate) fs_generator: &'a dyn ShaderGenerator,
    pub(crate) layout_generator: &'a dyn LayoutGenerator,
}

impl RenderContext {
    pub fn new() -> Self {
        RenderContext {
            shaders: HashMap::new(),
            pipelines: HashMap::new(),
        }
    }

    pub(crate) fn query_pipeline(
        &mut self,
        device: &wgpu::Device,
        query: &PipelineQuery,
    ) -> Rc<RefCell<Pipeline>> {
        let p = self.pipelines.get(&query.label).cloned();

        if p.is_some() {
            return p.unwrap();
        }

        let vs = self.get_shader(device, query.vs_generator);
        let fs = self.get_shader(device, query.fs_generator);

        let layout = query.layout_generator.generate(device);

        let pipeline = Rc::new(RefCell::new(Pipeline::new(
            query.label.clone(),
            vs,
            fs,
            layout,
            query.attributes.clone(),
            query.attr_stride,
        )));

        self.pipelines.insert(query.label.clone(), pipeline.clone());

        return pipeline;
    }

    fn get_shader(
        &mut self,
        device: &wgpu::Device,
        generator: &dyn ShaderGenerator,
    ) -> Rc<wgpu::ShaderModule> {
        let shader = self.shaders.get(&generator.shader_name()).cloned();

        if shader.is_some() {
            return shader.unwrap();
        }
        let shader = Rc::new(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(generator.shader_name().as_str()),
            source: wgpu::ShaderSource::Wgsl(generator.shader_code().into()),
        }));

        self.shaders.insert(generator.shader_name(), shader.clone());

        return shader;
    }
}
