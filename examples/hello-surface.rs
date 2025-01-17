mod common;

use common::App;

use mickey::{
    Color, FillRule, Paint, Path, Picture, PictureRecorder, Point, Rect, RenderContext, Surface,
};

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

        let rect = rect.offset(0.0, 200.0);

        recorder.save();
        recorder.rotate_at(rect.center(), 45.0);

        recorder.draw_rect(rect, paint);

        recorder.restore();

        let path = Path::new()
            .with_fill_rule(FillRule::EvenOdd)
            .move_to(Point::new(100.0, 10.0))
            .line_to(Point::new(40.0, 180.0))
            .line_to(Point::new(190.0, 60.0))
            .line_to(Point::new(10.0, 60.0))
            .line_to(Point::new(160.0, 180.0))
            .close_path();

        paint.color = Color::green().with_alpha(0.5);

        recorder.translate(300.0, 0.0);

        recorder.draw_path(path, paint);

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
            .with_clear_color(Color::white())
            .replay(self.picture.as_ref().expect("picture not init"))
            .flush(&mut self.context, device, queue);
        return false;
    }
}

fn main() {
    App::run("Hello world", 800, 800, HelloSurface::new());
}
