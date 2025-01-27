use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

pub trait Renderer {
    fn on_init(&mut self, format: wgpu::TextureFormat, device: &wgpu::Device, queue: &wgpu::Queue);

    fn on_render(
        &mut self,
        texture: &wgpu::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> bool;
}

pub struct App<'a, T: Renderer> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    surface: &'a wgpu::Surface<'a>,
    _format: wgpu::TextureFormat,
    window: &'a Window,

    renderer: T,
}

impl<'a, T: Renderer> App<'a, T> {
    pub fn run(title: &'static str, width: u32, height: u32, mut renderer: T) {
        let el = EventLoop::builder()
            .build()
            .expect("event loop creation failed");

        let wa = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height));
        let window = el.create_window(wa).expect("window creation failed");

        let instance = wgpu::Instance::default();

        let (adapter, device, queue) = App::<T>::request_device_and_queue(&instance);

        let surface = instance.create_surface(&window).unwrap();

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();

        config.format = wgpu::TextureFormat::Bgra8Unorm;

        surface.configure(&device, &config);

        renderer.on_init(config.format, &device, &queue);

        let mut app = App {
            device: &device,
            queue: &queue,
            surface: &surface,
            _format: config.format,
            window: &window,
            renderer,
        };

        let _ = el.run_app(&mut app);
    }

    fn request_device_and_queue(
        instance: &wgpu::Instance,
    ) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
        let adaptor = futures::executor::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            },
        ))
        .unwrap();

        let (device, queue) = futures::executor::block_on(adaptor.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))
        .unwrap();

        return (adaptor, device, queue);
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

impl<'a, T: Renderer> ApplicationHandler for App<'a, T> {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let texture = self.surface.get_current_texture().unwrap();

                let redraw = self
                    .renderer
                    .on_render(&texture.texture, self.device, self.queue);

                texture.present();

                if redraw {
                    self.request_redraw();
                }
            }
            _ => {}
        }
    }
}
