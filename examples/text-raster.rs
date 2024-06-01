mod common;

use std::rc::Rc;

use mickey::*;

struct TextRasterView {
    context: Option<GPUContext>,
    picture: Option<Picture>,
}

impl TextRasterView {
    fn new() -> Self {
        Self {
            context: None,
            picture: None,
        }
    }
}

fn gen_text_image(blob: Rc<TextBlob>) -> Rc<Bitmap> {
    Rc::new(blob.raster_to_image())
}

impl common::Renderer for TextRasterView {
    fn on_init(
        &mut self,
        _format: wgpu::TextureFormat,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.context = Some(GPUContext::new(device));

        let font = Font::new(
            FontDescription {
                name: "0xProtoNerdFont-Regular".to_string(),
                family: "0xProtoNerdFont".to_string(),
                style: FontStyle::normal(),
            },
            ab_glyph::FontArc::try_from_slice(include_bytes!(
                "./assets/0xProto/0xProtoNerdFont-Regular.ttf"
            ))
            .expect("Failed to load font"),
        );

        let builder = TextBlobBuilder::new(Rc::new(font), 60.0);

        let blob = builder.build("hello world j");

        println!(
            "blob size:[ {}, {} ]",
            blob.width.ceil(),
            blob.height.ceil()
        );
        println!("blob.ascent = {}", blob.ascent);
        println!("blob.descent = {}", blob.descent);
        println!("blob.line_gap = {}", blob.line_gap);

        let mut recorder = PictureRecorder::new();

        let image = gen_text_image(blob.clone());

        let width = image.info.width;
        let height = image.info.height;

        let rect = Rect::from_xywh(10.0, 10.0, width as f32, height as f32);

        recorder.draw_image(&Image::from_bitmap(image), &rect, None);

        let mut paint = Paint::new();
        paint.style = Style::Stroke(Stroke {
            width: 2.0,
            miter_limit: 4.0,
            ..Default::default()
        });
        paint.color = Color::red().into();

        recorder.draw_rect(&rect, &paint);

        let pos = Point::from(rect.left, rect.bottom);

        recorder.draw_text(blob.clone(), pos, Color::red());

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

        self.context.as_ref().unwrap().print_memory_usage();
    }
}

fn main() {
    let app = common::App::new("Text Raster", 800, 800);
    app.run(TextRasterView::new());
}
