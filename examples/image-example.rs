use std::{io::Cursor, rc::Rc};

use image::io::Reader as ImageReader;
use mickey::{core::*, gpu::GPUContext};

mod common;

struct ImageRender {
    pixel_buffer: Rc<Bitmap>,
    texture: Option<Rc<wgpu::Texture>>,

    context: Option<GPUContext>,
    picture: Option<Picture>,
}

impl ImageRender {
    fn new() -> Self {
        let file = include_bytes!("assets/mandrill.png");
        let ret = ImageReader::new(Cursor::new(file))
            .with_guessed_format()
            .expect("Invalid image")
            .decode()
            .expect("Decode error");
        println!("image color: {:?}", ret.color());
        println!("image bytes: {}", ret.as_bytes().len());
        println!("image width: {}, height: {}", ret.width(), ret.height());

        let pixel = ret.into_rgba8();

        Self {
            pixel_buffer: Rc::new(Bitmap::new(
                ImageInfo {
                    width: pixel.width(),
                    height: pixel.height(),
                    format: ImageFormat::RGBA8888,
                    premultiplied: true,
                },
                pixel.into_vec(),
                None,
            )),
            texture: None,
            context: None,
            picture: None,
        }
    }
}

fn gen_texture(
    bitmap: Rc<Bitmap>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Rc<wgpu::Texture> {
    let width = bitmap.info.width;
    let height = bitmap.info.height;
    let format = match bitmap.info.format {
        ImageFormat::RGBA8888 => wgpu::TextureFormat::Rgba8Unorm,
        ImageFormat::BGRA8888 => wgpu::TextureFormat::Bgra8Unorm,
        ImageFormat::RGBX8888 => wgpu::TextureFormat::Rgba8Unorm,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[format],
    });

    queue.write_texture(
        wgpu::ImageCopyTextureBase {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            aspect: Default::default(),
        },
        &bitmap.data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(bitmap.bytes_per_row),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    return Rc::new(texture);
}

impl common::Renderer for ImageRender {
    fn on_init(
        &mut self,
        _format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.context = Some(GPUContext::new(device));
        self.texture = Some(gen_texture(self.pixel_buffer.clone(), device, queue));

        let mut recorder = PictureRecorder::new();

        let image = Image::from_bitmap(self.pixel_buffer.clone());

        recorder.draw_image(&image, &Rect::from_xywh(100.0, 100.0, 300.0, 300.0), None);

        let tex_image = Image::from_texture(
            self.texture.as_ref().unwrap().clone(),
            ImageInfo {
                width: image.width(),
                height: image.height(),
                format: ImageFormat::RGBA8888,
                premultiplied: true,
            },
        );

        recorder.draw_image(
            &tex_image,
            &Rect::from_xywh(450.0, 100.0, 300.0, 300.0),
            None,
        );

        recorder.draw_image(
            &tex_image,
            &Rect::from_xywh(100.0, 450.0, 300.0, 300.0),
            Some(&Rect::from_xywh(50.0, 40.0, 300.0, 300.0)),
        );

        recorder.draw_image(
            &tex_image,
            &Rect::from_xywh(450.0, 450.0, 300.0, 300.0),
            Some(&Rect::from_xywh(200.0, 200.0, 300.0, 300.0)),
        );

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
    let app = common::App::new("Image Example", 800, 800);
    app.run(ImageRender::new());
}
