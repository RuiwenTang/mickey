/// The render context. Which used to cache the pipeline and other resources.
/// Created by the rendering engine. And shared the context to all the surfaces.
/// Can help reduce the resource usage.
pub struct RenderContext {}

impl RenderContext {
    pub fn new() -> Self {
        RenderContext {}
    }
}
