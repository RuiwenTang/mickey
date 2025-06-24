#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use winit::{
    dpi::PhysicalSize, platform::web::WindowAttributesExtWebSys, platform::web::WindowExtWebSys,
};

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
    adapter: &'a wgpu::Adapter,
    instance: &'a wgpu::Instance,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    wa: WindowAttributes,
    window: Option<Window>,
    surface: Option<wgpu::Surface<'a>>,

    renderer: T,
}

impl<'a, T: Renderer> App<'a, T> {
    pub async fn run(title: &'static str, width: u32, height: u32, renderer: T) {
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        #[cfg(not(target_arch = "wasm32"))]
        let el = EventLoop::builder()
            .build()
            .expect("event loop creation failed");

        #[cfg(target_arch = "wasm32")]
        let el = EventLoop::new().expect("event loop creation failed");

        el.set_control_flow(winit::event_loop::ControlFlow::Poll);

        #[cfg(not(target_arch = "wasm32"))]
        let wa = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height));

        #[cfg(target_arch = "wasm32")]
        let wa = WindowAttributes::default()
            .with_inner_size(LogicalSize::new(width, height))
            .with_append(true);

        let instance = wgpu::Instance::default();

        let (adapter, device, queue) = App::<T>::request_device_and_queue(&instance).await;

        let mut app = App {
            adapter: &adapter,
            instance: &instance,
            device: &device,
            queue: &queue,
            surface: None,
            wa,
            window: None,
            renderer,
        };

        let _ = el.run_app(&mut app);
    }

    async fn request_device_and_queue(
        instance: &wgpu::Instance,
    ) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
        let adaptor = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        let (device, queue) = adaptor
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        return (adaptor, device, queue);
    }

    pub fn request_redraw(&self) {
        self.window
            .as_ref()
            .expect("window not created")
            .request_redraw();
    }
}

impl<'a, T: Renderer> ApplicationHandler for App<'a, T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(event_loop.create_window(self.wa.clone()).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        let size = self
            .window
            .as_ref()
            .expect("window not created")
            .inner_size();

        #[cfg(target_arch = "wasm32")]
        let size = self
            .wa
            .inner_size
            .unwrap()
            .to_physical(self.window.as_ref().unwrap().scale_factor());

        #[cfg(target_arch = "wasm32")]
        {
            let message = format!("width: {}, height: {}", size.width, size.height);
            web_sys::console::log_1(&message.into());
        }

        let surface = unsafe {
            self.instance
                .create_surface_unsafe(
                    wgpu::SurfaceTargetUnsafe::from_window(
                        self.window.as_ref().expect("window not created"),
                    )
                    .expect("failed create surface target"),
                )
                .expect("failed to create surface")
        };

        let mut config = surface
            .get_default_config(self.adapter, size.width, size.height)
            .unwrap();

        config.format = wgpu::TextureFormat::Bgra8Unorm;

        surface.configure(self.device, &config);

        self.surface = Some(surface);

        self.renderer
            .on_init(config.format, self.device, self.queue);

        #[cfg(target_arch = "wasm32")]
        self.window.as_ref().unwrap().request_redraw();
    }

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
                let texture = self
                    .surface
                    .as_ref()
                    .expect("surface not init")
                    .get_current_texture()
                    .unwrap();

                let redraw = self
                    .renderer
                    .on_render(&texture.texture, self.device, self.queue);

                texture.present();

                if redraw {
                    self.request_redraw();
                }

                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&"redraw done".into());
            }
            _ => {}
        }
    }
}
