mod common;

use std::rc::Rc;

use rskity::core::Surface as GPUSurface;
use rskity::gpu::GPUContext;

struct SurfaceExample {
    context: Rc<GPUContext>,
}

impl SurfaceExample {
    fn new() -> Self {
        Self {
            context: Rc::new(GPUContext::new()),
        }
    }
}

impl common::Renderer for SurfaceExample {
    fn on_init(&mut self, format: wgpu::TextureFormat, device: &wgpu::Device, queue: &wgpu::Queue) {
    }

    fn on_render(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) {
        let text = surface.get_current_texture();

        if text.is_err() {
            return;
        }

        let text = text.unwrap();

        let mut surface = GPUSurface::new(&text.texture, false, device);
        surface.flush(
            &mut self.context,
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
    let app = common::App::new("Surface Example", 800, 800);
    app.run(SurfaceExample::new());
}
