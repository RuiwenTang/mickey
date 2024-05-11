use rskity::{core::*, gpu::GPUContext};

mod common;

/// same as https://fiddle.skia.org/c/@shapes
fn draw_shapes(recorder: &mut PictureRecorder) {
    let mut paint = Paint::new();

    paint.color = Color::from_rgba_u8(0x42, 0x85, 0xF4, 0xFF);
    let rect = Rect::from_xywh(10.0, 10.0, 100.0, 160.0);
    recorder.draw_rect(&rect, paint);

    let mut oval = RRect::new_oval(rect);
    oval.offset(40.0, 80.0);
    paint.color = Color::from_rgba_u8(0xDB, 0x44, 0x37, 0xFF);
    recorder.draw_rrect(&oval, paint);

    paint.color = Color::from_rgba_u8(0x0F, 0x9D, 0x58, 0xFF);
    recorder.draw_circle(180.0, 50.0, 25.0, paint);

    paint.style = Stroke::default().with_width(4.0).into();
    paint.color = Color::from_rgba_u8(0xF4, 0xB4, 0x0, 0xFF);

    let mut rrect = RRect::from_rect_xy(rect, 10.0, 10.0);
    rrect.offset(80.0, 50.0);
    recorder.draw_rrect(&rrect, paint);
}

/// same as https://fiddle.skia.org/c/@Canvas_drawCircle
fn draw_circle(recorder: &mut PictureRecorder) {
    let mut paint = Paint::new();
    paint.style = Style::Fill;
    paint.color = Color::black();
    recorder.draw_circle(128.0, 128.0, 90.0, paint);
    paint.color = Color::white();
    recorder.draw_circle(86.0, 86.0, 20.0, paint);
    recorder.draw_circle(160.0, 76.0, 20.0, paint);
    recorder.draw_circle(140.0, 150.0, 35.0, paint);
}

/// same as https://fiddle.skia.org/c/@Canvas_drawRoundRect
fn draw_round_rect(recorder: &mut PictureRecorder) {
    let fill_paint = Paint::new();
    let mut stroke_paint = Paint::new();

    stroke_paint.style = Stroke::default()
        .with_width(15.0)
        .with_join(StrokeJoin::Round)
        .into();

    let radii = [
        Point::from(0.0, 20.0),
        Point::from(10.0, 10.0),
        Point::from(10.0, 20.0),
        Point::from(10.0, 40.0),
    ];

    recorder.save();

    for i in 0..2 {
        let p = if i == 0 { &stroke_paint } else { &fill_paint };

        for rad in &radii {
            recorder.draw_rrect(
                &RRect::from_rect_xy(Rect::from_ltrb(10.0, 10.0, 60.0, 40.0), rad.x, rad.y),
                *p,
            );
            recorder.translate(0.0, 60.0);
        }

        recorder.translate(80.0, -240.0);
    }

    recorder.restore();
}

/// same as https://fiddle.skia.org/c/@Canvas_clipPath_2
fn draw_clip_path(recorder: &mut PictureRecorder) {
    let paint = Paint::new();

    let mut path = Path::new()
        .add_rect(&Rect::from_ltrb(20.0, 15.0, 100.0, 95.0))
        .add_rect(&Rect::from_ltrb(50.0, 65.0, 130.0, 135.0));

    recorder.save();

    recorder.clip_path(path.clone(), ClipOp::Intersect);

    recorder.draw_circle(70.0, 85.0, 60.0, paint);

    recorder.restore();

    recorder.save();

    recorder.translate(100.0, 100.0);

    path.fill_type = PathFillType::EvenOdd;

    recorder.clip_path(path, ClipOp::Intersect);
    recorder.draw_circle(70.0, 85.0, 60.0, paint);

    recorder.restore();
}

/// same as https://fiddle.skia.org/c/@Canvas_clipRect_2
fn draw_clip_rect(recorder: &mut PictureRecorder) {
    let paint = Paint::new();

    let rect = Rect::from_xywh(0.0, 0.0, 90.0, 120.0);

    recorder.save();

    recorder.clip_rect(&rect, ClipOp::Intersect);
    recorder.draw_circle(100.0, 100.0, 60.0, paint);

    recorder.restore();

    recorder.save();

    recorder.translate(80.0, 0.0);

    recorder.clip_rect(&rect, ClipOp::Difference);
    recorder.draw_circle(100.0, 100.0, 60.0, paint);

    recorder.restore();
}

struct ShapeRender {
    context: Option<GPUContext>,
    picture: Option<Picture>,
}

impl ShapeRender {
    fn new() -> Self {
        Self {
            context: None,
            picture: None,
        }
    }

    fn render_shape(&self) -> Picture {
        let mut recorder = PictureRecorder::new();

        draw_shapes(&mut recorder);

        recorder.save();
        recorder.translate(200.0, 0.0);

        draw_circle(&mut recorder);

        recorder.restore();

        recorder.save();
        recorder.translate(500.0, 0.0);

        draw_round_rect(&mut recorder);

        recorder.restore();

        recorder.save();
        recorder.translate(0.0, 300.0);
        draw_clip_path(&mut recorder);
        recorder.restore();

        recorder.save();
        recorder.translate(300.0, 300.0);
        draw_clip_rect(&mut recorder);
        recorder.restore();

        return recorder.finish_record();
    }
}

impl common::Renderer for ShapeRender {
    fn on_init(
        &mut self,
        _format: wgpu::TextureFormat,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.context = Some(GPUContext::new(device));
        self.picture = Some(self.render_shape());
    }

    fn on_render(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) {
        let text = surface.get_current_texture();

        if text.is_err() {
            return;
        }

        let text = text.unwrap();

        let mut surface = Surface::new(&text.texture, 800.0, 800.0, true, device);

        surface.replay(self.picture.as_ref().unwrap());

        surface.flush(
            &mut self.context.as_mut().unwrap(),
            device,
            queue,
            Some(wgpu::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
        );

        text.present();
    }
}

fn main() {
    let app = common::App::new("Shape Example", 800, 800);
    app.run(ShapeRender::new());
}
