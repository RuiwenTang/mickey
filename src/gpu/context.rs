use std::collections::HashMap;

use crate::render::fragment::ColorPipelineGenerator;

use super::pipeline::Pipeline;

pub(crate) trait PipelineGenerater {
    fn label(&self) -> &'static str;

    fn gen_pipeline(&self, format: wgpu::TextureFormat, device: &wgpu::Device) -> Pipeline;
}

struct PipelineNode {
    format: wgpu::TextureFormat,
    pipelines: HashMap<&'static str, Pipeline>,
}

impl PipelineNode {
    pub(crate) fn new(format: wgpu::TextureFormat) -> Self {
        PipelineNode {
            format,
            pipelines: HashMap::new(),
        }
    }

    pub(crate) fn load_pipeline(
        &mut self,
        label: &'static str,
        generator: &Box<dyn PipelineGenerater>,
        device: &wgpu::Device,
    ) {
        if self.pipelines.contains_key(label) {
            return;
        }

        self.pipelines
            .insert(label, generator.gen_pipeline(self.format, device));
    }

    pub(crate) fn get_pipeline(&self, label: &'static str) -> Option<&Pipeline> {
        self.pipelines.get(label)
    }
}

/// GPU context for holding pipelines created by engine
pub struct GPUContext {
    pipelines: HashMap<wgpu::TextureFormat, PipelineNode>,

    generator: HashMap<&'static str, Box<dyn PipelineGenerater>>,
}

impl GPUContext {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut generator: HashMap<&'static str, Box<dyn PipelineGenerater>> = HashMap::new();

        generator.insert(
            "SolidColor",
            ColorPipelineGenerator::solid_color_pipeline(device),
        );

        Self {
            pipelines: HashMap::new(),
            generator,
        }
    }

    pub(crate) fn load_pipeline(
        &mut self,
        label: &'static str,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
    ) {
        let pg = self.generator.get(label);

        if pg.is_none() {
            return;
        }

        let pg = pg.unwrap();

        let p = self
            .pipelines
            .entry(format)
            .or_insert(PipelineNode::new(format));

        p.load_pipeline(label, pg, device);
    }

    pub(crate) fn get_pipeline(
        &self,
        label: &'static str,
        format: wgpu::TextureFormat,
    ) -> Option<&Pipeline> {
        let node = self.pipelines.get(&format);

        if node.is_none() {
            return None;
        }

        let node = node.unwrap();

        node.get_pipeline(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::init_test_context;

    #[test]
    fn test_context() {
        let (device, _queue) = init_test_context();

        let mut ctx = GPUContext::new(&device);

        ctx.load_pipeline("SolidColor", wgpu::TextureFormat::Rgba8Unorm, &device);

        assert!(ctx
            .get_pipeline("SolidColor", wgpu::TextureFormat::Bgra8Unorm)
            .is_none());
        assert!(ctx
            .get_pipeline("SolidColor", wgpu::TextureFormat::Rgba8Unorm)
            .is_some());
    }
}
