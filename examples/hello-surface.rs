mod common;

use common::App;

use mickey::*;

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

        recorder.draw_rect(rect, &paint);

        paint.set_color(Color::blue().with_alpha(0.3));

        let rect = rect.offset(0.0, 200.0);

        recorder.save();
        recorder.rotate_at(rect.center(), 45.0.degree());

        paint.set_style(Stroke::default().with_width(10.0));

        recorder.draw_rect(rect, &paint);

        recorder.restore();

        let path = Path::new()
            .with_fill_rule(FillRule::EvenOdd)
            .move_to((100.0, 10.0))
            .line_to((40.0, 180.0))
            .line_to((190.0, 60.0))
            .line_to((10.0, 60.0))
            .line_to((160.0, 180.0))
            .close_path();

        paint.set_color(Color::green().with_alpha(0.5));
        paint.set_style(Style::Fill);

        recorder.save();
        recorder.translate(300.0, 0.0);

        recorder.draw_path(path, &paint);

        recorder.draw_circle((300.0, 300.0), 100.0, &paint);
        recorder.restore();

        paint.set_color(Color::red().with_alpha(0.5));

        paint.set_style(
            Stroke::default()
                .with_cap(StrokeCap::Round)
                .with_join(StrokeJoin::Round)
                .with_width(10.0),
        );

        recorder.save();
        recorder.translate(600.0, 100.0);
        recorder.draw_path(
            Path::new()
                .move_to((10.0, 10.0))
                .quad_to((256.0, 64.0), (128.0, 128.0))
                .quad_to((10.0, 192.0), (250.0, 250.0))
                .close_path(),
            &paint,
        );
        recorder.restore();

        {
            recorder.save();
            recorder.translate(600.0, 400.0);

            let path = Path::new()
                .add_rect(&Rect::new_ltrb(20.0, 15.0, 100.0, 95.0))
                .add_rect(&Rect::new_ltrb(50.0, 65.0, 130.0, 135.0));

            recorder.save();
            recorder.clip(path.clone(), Default::default());

            recorder.draw_circle((70.0, 85.0), 60.0, &Paint::default());
            recorder.restore();

            recorder.translate(100.0, 100.0);

            recorder.clip(
                path.clone().with_fill_rule(FillRule::EvenOdd),
                ClipOp::Difference,
            );
            recorder.draw_circle((70.0, 85.0), 60.0, &Paint::default());

            recorder.restore();
        }

        {
            recorder.save();
            recorder.translate(100.0, 400.0);
            let mut paint = Paint::default();
            paint.set_color(RadialGradient::new(Point::new(220.0, 350.0), 150.0, vec![
                Color::white(),
                Color::black(),
            ]));

            recorder.draw_circle((220.0, 350.0), 100.0, &paint);
            recorder.restore();
        }

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
            .with_view_port(Rect::new_xywh(0.0, 0.0, 1000.0, 1000.0))
            .replay(self.picture.as_ref().expect("picture not init"))
            .flush(&mut self.context, device, queue);
        return false;
    }
}

fn main() {
    App::run("Hello world", 800, 800, HelloSurface::new());
}
