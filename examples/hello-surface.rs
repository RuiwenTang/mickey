mod common;

use std::rc::Rc;

use common::App;

use mickey::{Color, Paint, Picture, PictureRecorder, Rect, RenderContext, Surface};

struct HelloSurface {
    context: RenderContext,
    picture: Option<Picture>,
}

impl HelloSurface {
    fn new() -> Self {
        HelloSurface {
            context: RenderContext::new(),
            picture: None,
        }
    }
}

impl common::Renderer for HelloSurface {
    fn on_init(
        &mut self,
        _format: wgpu::TextureFormat,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        let mut recorder = PictureRecorder::new();

        let mut paint = Paint::default().with_color(Color::red());
        let rect = Rect::new_xywh(100.0, 100.0, 100.0, 100.0);

        recorder.draw_rect(rect, paint);

        paint.set_color(Color::blue());

        recorder.draw_rect(rect.offset(0.0, 200.0), paint);

        self.picture = Some(recorder.finish_recorder());
    }

    fn on_render(
        &mut self,
        texture: &wgpu::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> bool {
        let surface = Surface::new(texture);

        surface
            .with_clear_color(Color::cyan())
            .replay(self.picture.as_ref().expect("picture not init"))
            .flush(&mut self.context, device, queue);
        return false;
    }
}

fn main() {
    App::run("Hello world", 800, 800, HelloSurface::new());
}
