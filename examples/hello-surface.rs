mod common;
use common::App;

use mickey::{Color, Surface as MCSurface};

struct HelloSurface {}

impl HelloSurface {
    fn new() -> Self {
        HelloSurface {}
    }
}

impl common::Renderer for HelloSurface {
    fn on_init(
        &mut self,
        _format: wgpu::TextureFormat,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }

    fn on_render(
        &mut self,
        texture: &wgpu::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> bool {
        let surface = MCSurface::new(texture);

        surface.with_clear_color(Color::cyan()).flush(device, queue);
        return false;
    }
}

fn main() {
    App::run("Hello world", 800, 800, HelloSurface::new());
}
