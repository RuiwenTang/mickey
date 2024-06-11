use mickey::*;

mod common;

struct NanovgRender {
    width: f32,
    height: f32,
    context: Option<GPUContext>,
}

impl NanovgRender {
    fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            context: None,
        }
    }

    fn render(&self) -> Picture {
        let mut recorder = PictureRecorder::new();

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
