pub(crate) mod atlas;
pub(crate) mod buffer;
pub(crate) mod context;
pub(crate) mod pipeline;

pub use context::GPUContext;

/// Only for test
#[cfg(test)]
pub(crate) fn init_test_context() -> (wgpu::Device, wgpu::Queue) {
    static CALL_ONCE: std::sync::Once = std::sync::Once::new();

    CALL_ONCE.call_once(|| {
        env_logger::init_from_env(env_logger::Env::default().default_filter_or("error"));
    });

    let instance = wgpu::Instance::default();

    let adapter =
        futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .unwrap();

    let (device, queue) = futures::executor::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("test device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap();

    return (device, queue);
}
