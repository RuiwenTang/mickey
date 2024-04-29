use winit::{
    dpi::{LogicalSize, Size},
    event_loop::EventLoop,
    window::WindowBuilder,
};

pub trait Renderer {
    fn on_init(&mut self, format: wgpu::TextureFormat, device: &wgpu::Device, queue: &wgpu::Queue);

    fn on_render(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue);
}

pub struct App {
    title: &'static str,
    width: u32,
    height: u32,
}

impl App {
    pub fn new(title: &'static str, width: u32, height: u32) -> Self {
        App {
            title,
            width,
            height,
        }
    }

    pub fn run<T: Renderer>(&self, mut render: T) {
        let el = EventLoop::new().unwrap();

        let wb = WindowBuilder::new();
        let window = wb
            .with_title(self.title)
            .with_inner_size(Size::Logical(LogicalSize {
                width: self.width as f64,
                height: self.height as f64,
            }))
            .build(&el)
            .unwrap();

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(&window).unwrap();

        let (adapter, device, queue) = self.request_device_and_queue(&instance, &surface);

        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();

        surface.configure(&device, &config);

        render.on_init(config.format, &device, &queue);

        let _ = el.run(|e, elwt| match e {
            winit::event::Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                winit::event::WindowEvent::RedrawRequested => {
                    render.on_render(&surface, &device, &queue);
                }
                _ => {}
            },
            _ => {}
        });
    }

    fn request_device_and_queue(
        &self,
        instance: &wgpu::Instance,
        surface: &wgpu::Surface,
    ) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
        let adaptor = futures::executor::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            },
        ))
        .unwrap();

        let (device, queue) = futures::executor::block_on(adaptor.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        return (adaptor, device, queue);
    }
}
