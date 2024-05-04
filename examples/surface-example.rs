mod common;

use rskity::core::{Color, Paint, Point, Surface as GPUSurface};
use rskity::core::{Path, PathFillType, Picture, PictureRecorder};
use rskity::gpu::GPUContext;

struct SurfaceExample {
    context: Option<GPUContext>,
    picture: Option<Picture>,
}

impl SurfaceExample {
    fn new() -> Self {
        Self {
            context: None,
            picture: None,
        }
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

        let mut path = Path::new(PathFillType::Winding)
            .move_to(100.0, 10.0)
            .line_to(40.0, 180.0)
            .line_to(190.0, 60.0)
            .line_to(10.0, 60.0)
            .line_to(160.0, 180.0)
            .close();

        let mut recorder = PictureRecorder::new();

        let mut paint = Paint::new();
        paint.color = Color::red().with_alpha(0.5);

        recorder.draw_path(path.clone(), paint.clone());

        recorder.translate(200.0, 0.0);

        path.fill_type = PathFillType::EvenOdd;

        paint.color = Color::magenta();

        recorder.draw_path(path, paint.clone());

        let curve = Path::new(PathFillType::Winding)
            .move_to(10.0, 10.0)
            .quad_to_point(Point { x: 256.0, y: 64.0 }, Point { x: 128.0, y: 128.0 })
            .close();

        recorder.translate(200.0, 0.0);

        recorder.draw_path(curve, paint.clone());

        self.picture = Some(recorder.finish_record());
    }

    fn on_render(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) {
        let text = surface.get_current_texture();

        if text.is_err() {
            return;
        }

        let text = text.unwrap();

        let mut surface = GPUSurface::new(&text.texture, 800.0, 800.0, true, device);

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
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
    let app = common::App::new("Surface Example", 800, 800);
    app.run(SurfaceExample::new());
}
