mod common;

use rskity::core::Surface as GPUSurface;
use rskity::gpu::GPUContext;

struct SurfaceExample {
    context: Option<GPUContext>,
}

impl SurfaceExample {
    fn new() -> Self {
        Self { context: None }
    }
}

impl common::Renderer for SurfaceExample {
    fn on_init(
        &mut self,
        _format: wgpu::TextureFormat,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.context = Some(GPUContext::new(device));
    }

    fn on_render(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) {
        let text = surface.get_current_texture();

        if text.is_err() {
            return;
        }

        let text = text.unwrap();

        let mut surface = GPUSurface::new(&text.texture, false, device);
        surface.flush(
            &mut self.context.as_mut().unwrap(),
            device,
            queue,
            Some(wgpu::Color {
                r: 1.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }),
        );

        text.present();
    }
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
    let app = common::App::new("Surface Example", 800, 800);
    app.run(SurfaceExample::new());
}
