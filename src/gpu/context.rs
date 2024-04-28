use std::collections::HashMap;

use super::pipeline::Pipeline;

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
}
