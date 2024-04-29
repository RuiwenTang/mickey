use std::collections::HashMap;

use super::pipeline::Pipeline;

pub(crate) trait PipelineGenerater {
    fn generate_pipeline(&self, device: &wgpu::Device) -> Pipeline;
}

/// GPU context for holding pipelines created by engine
pub struct GPUContext {
    pub(crate) pipelines: HashMap<&'static str, Pipeline>,
}

impl GPUContext {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    pub fn get_pipeline(&self, key: &'static str) -> Option<&Pipeline> {
        self.pipelines.get(key)
    }

    pub(crate) fn get_or_create_pipeline<T: PipelineGenerater>(
        &mut self,
        device: &wgpu::Device,
        key: &'static str,
        generater: &T,
    ) -> &Pipeline {
        return self
            .pipelines
            .entry(key)
            .or_insert_with(|| generater.generate_pipeline(device));
    }
}
