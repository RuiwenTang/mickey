use std::time;

use mickey::*;

mod common;

struct NanovgRender {
    begin_time: time::Instant,
    width: f32,
    height: f32,
    context: Option<GPUContext>,
}

impl NanovgRender {
    fn new(width: f32, height: f32) -> Self {
        Self {
            begin_time: time::Instant::now(),
            width,
            height,
            context: None,
        }
    }

    fn draw_eyes(
        &self,
        recorder: &mut PictureRecorder,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        mx: f32,
        my: f32,
        t: f64,
    ) {
        let ex = w * 0.23;
        let ey = h * 0.5;
        let lx = x + ex;
        let ly = y + ey;
        let rx = x + w - ex;
        let ry = y + ey;
        let br = if ex < ey { ex * 0.5 } else { ey * 0.5 };
        let blink = 1.0 - (t * 0.5).sin().powf(200.0) * 0.8;

        let mut bg = Paint::new();
        bg.style = Style::Fill;
        bg.color =
            LinearGradient::new(Point::from(x, y + h * 0.5), Point::from(x + w * 0.1, y + h))
                .with_colors(vec![
                    Color::from_rgba_u8(0, 0, 0, 32),
                    Color::from_rgba_u8(0, 0, 0, 16),
                ])
                .into();

        recorder.draw_oval(
            &Rect::from_ltrb(lx + 0.3 - ex, ly + 16.0 - ey, lx + 3.0 + ex, ly + 16.0 + ey),
            &bg,
        );

        recorder.draw_oval(
            &Rect::from_ltrb(rx + 3.0 - ex, ry + 16.0 - ey, rx + 3.0 + ex, ry + 16.0 + ey),
            &bg,
        );

        bg.color = LinearGradient::new(
            Point::from(x, y + h * 0.25),
            Point::from(x + w * 0.1, y + h),
        )
        .with_colors(vec![
            Color::from_rgba_u8(220, 220, 220, 255),
            Color::from_rgba_u8(128, 128, 128, 255),
        ])
        .into();

        recorder.draw_oval(&Rect::from_ltrb(lx - ex, ly - ey, lx + ex, ly + ey), &bg);
        recorder.draw_oval(&Rect::from_ltrb(rx - ex, ry - ey, rx + ex, ry + ey), &bg);
    }

    fn render(&self) -> Picture {
        let current = time::Instant::now();

        let delta = (current - self.begin_time).as_millis() as f64;

        let mut recorder = PictureRecorder::new();

        self.draw_eyes(
            &mut recorder,
            self.width - 250.0,
            50.0,
            150.0,
            100.0,
            0.0,
            0.0,
            delta,
        );

        return recorder.finish_record();
    }
}

impl common::Renderer for NanovgRender {
    fn on_init(
        &mut self,
        _format: wgpu::TextureFormat,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.context = Some(GPUContext::new(device));
    }

    fn on_render(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) {
        let text = surface
            .get_current_texture()
            .expect("can not acquire texture");

        let mut surface = GPUSurface::new(&text.texture, self.width, self.height, true, device);

        let picture = self.render();

        surface.replay(&picture);

        surface.flush(
            self.context.as_mut().unwrap(),
            device,
            queue,
            Some(wgpu::Color {
                r: 0.3,
                g: 0.3,
                b: 0.32,
                a: 1.0,
            }),
        );

        text.present();
    }
}

fn main() {
    let width = 1000.0;
    let height = 600.0;
    let app = common::App::new("NanoVG Frame Example", width as u32, height as u32, true);

    app.run(NanovgRender::new(width, height));
}
