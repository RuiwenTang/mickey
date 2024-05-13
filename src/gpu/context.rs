use std::collections::HashMap;

use crate::render::fragment::ColorPipelineGenerator;

use super::pipeline::Pipeline;

pub(crate) trait PipelineGenerater {
    fn gen_pipeline(
        &self,
        format: wgpu::TextureFormat,
        sample_count: u32,
        device: &wgpu::Device,
    ) -> Pipeline;
}

struct PipelineNode {
    format: wgpu::TextureFormat,
    sample_count: u32,
    pipelines: HashMap<&'static str, Pipeline>,
}

impl PipelineNode {
    pub(crate) fn new(format: wgpu::TextureFormat, sample_count: u32) -> Self {
        PipelineNode {
            format,
            sample_count,
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

        self.pipelines.insert(
            label,
            generator.gen_pipeline(self.format, self.sample_count, device),
        );
    }

    pub(crate) fn get_pipeline(&self, label: &'static str) -> Option<&Pipeline> {
        self.pipelines.get(label)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct PipelineKey {
    format: wgpu::TextureFormat,
    sample_count: u32,
}

/// GPU context for holding pipelines created by engine. Only one context is needed.
pub struct GPUContext {
    pipelines: HashMap<PipelineKey, PipelineNode>,

    generator: HashMap<&'static str, Box<dyn PipelineGenerater>>,
}

impl GPUContext {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut generator: HashMap<&'static str, Box<dyn PipelineGenerater>> = HashMap::new();

        generator.insert(
            "SolidColor",
            ColorPipelineGenerator::solid_color_pipeline(device),
        );

        generator.insert(
            "LinearGradient",
            ColorPipelineGenerator::linear_gradient_pipeline(device),
        );

        generator.insert(
            "NonColor",
            ColorPipelineGenerator::non_color_pipeline(device),
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
        anti_aliasing: bool,
        device: &wgpu::Device,
    ) {
        let pg = self.generator.get(label);

        if pg.is_none() {
            return;
        }

        let pg = pg.unwrap();

        let p = self
            .pipelines
            .entry(PipelineKey {
                format,
                sample_count: if anti_aliasing { 4 } else { 1 },
            })
            .or_insert(PipelineNode::new(format, if anti_aliasing { 4 } else { 1 }));

        p.load_pipeline(label, pg, device);
    }

    pub(crate) fn get_pipeline(
        &self,
        label: &'static str,
        format: wgpu::TextureFormat,
        anti_alias: bool,
    ) -> Option<&Pipeline> {
        let node = self.pipelines.get(&PipelineKey {
            format,
            sample_count: if anti_alias { 4 } else { 1 },
        });

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
    use crate::{
        gpu::init_test_context,
        render::fragment::{
            LINEAR_GRADIENT_PIPELINE_NAME, NON_COLOR_PIPELINE_NAME, SOLID_PIPELINE_NAME,
        },
    };

    #[test]
    fn test_context() {
        let (device, _queue) = init_test_context();

        let mut ctx = GPUContext::new(&device);

        ctx.load_pipeline(
            SOLID_PIPELINE_NAME,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            &device,
        );

        ctx.load_pipeline(
            LINEAR_GRADIENT_PIPELINE_NAME,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            &device,
        );

        ctx.load_pipeline(
            NON_COLOR_PIPELINE_NAME,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            &device,
        );

        assert!(ctx
            .get_pipeline(SOLID_PIPELINE_NAME, wgpu::TextureFormat::Bgra8Unorm, false)
            .is_none());
        assert!(ctx
            .get_pipeline(SOLID_PIPELINE_NAME, wgpu::TextureFormat::Rgba8Unorm, false)
            .is_some());

        assert!(ctx
            .get_pipeline(
                NON_COLOR_PIPELINE_NAME,
                wgpu::TextureFormat::Rgba8Unorm,
                false
            )
            .is_some());
    }
}
