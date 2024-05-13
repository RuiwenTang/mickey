use rskity::{core::*, gpu::GPUContext};

mod common;

struct GradientRender {
    context: Option<GPUContext>,
    picture: Option<Picture>,
}

impl GradientRender {
    fn new() -> Self {
        Self {
            context: None,
            picture: None,
        }
    }
}

fn draw_basic_gradient(recorder: &mut PictureRecorder) {
    let rect = Rect::from_xywh(10.0, 10.0, 200.0, 200.0);

    let mut paint = Paint::new();
    paint.style = Style::Fill;
    paint.color = LinearGradient::new(
        Point::from(rect.left, rect.top),
        Point::from(rect.right, rect.bottom),
    )
    .with_colors_stops(
        vec![Color::red(), Color::green(), Color::blue()],
        vec![0.0, 0.3, 1.0],
    )
    .into();

    recorder.draw_rect(&rect, &paint);
}

impl common::Renderer for GradientRender {
    fn on_init(
        &mut self,
        _format: wgpu::TextureFormat,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.context = Some(GPUContext::new(device));

        let mut recorder = PictureRecorder::new();

        draw_basic_gradient(&mut recorder);

        self.picture = Some(recorder.finish_record());
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
    let app = common::App::new("Gradient Example", 800, 800);
    app.run(GradientRender::new());
}
