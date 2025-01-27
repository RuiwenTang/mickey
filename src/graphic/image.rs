use std::rc::Rc;

use crate::{PaintColor, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFormat {
    #[default]
    RGBA8888,
    BGRA8888,
    RGBX8888,
    BGRX8888,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub premultiplied: bool,
}

/// A bitmap image. The image is stored in a contiguous block of memory.
#[derive(Debug, Clone, PartialEq)]
pub struct Bitmap {
    pub info: ImageInfo,
    pub data: Vec<u8>,
    pub bytes_per_row: u32,
}

/// The source of an image.
/// The source can be either a [`Bitmap`] or a [`wgpu::Texture`].
#[derive(Debug, Clone, PartialEq)]
pub enum ImageSource {
    Bitmap(Rc<Bitmap>),
    Texture(Rc<wgpu::Texture>, ImageInfo),
}

/// An image that can be used as a source for drawing.
/// The image can be either a [`Bitmap`] or a [`wgpu::Texture`].
///
/// # Note
/// If pass a [`Bitmap`] as the source, the image will be uploaded to the GPU as a texture every time it is drawn.
/// If you want to avoid this, you should create a [`wgpu::Texture`] from the [`Bitmap`] and pass it as the source.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub(crate) source: ImageSource,
}

impl ImageInfo {
    pub fn new(width: u32, height: u32, format: ImageFormat, premultiplied: bool) -> Rc<Self> {
        Rc::new(Self {
            width,
            height,
            format,
            premultiplied,
        })
    }
}

impl Image {
    pub fn get_width(&self) -> u32 {
        match &self.source {
            ImageSource::Bitmap(bitmap) => bitmap.info.width,
            ImageSource::Texture(texture, _) => texture.width(),
        }
    }

    pub fn get_height(&self) -> u32 {
        match &self.source {
            ImageSource::Bitmap(bitmap) => bitmap.info.height,
            ImageSource::Texture(texture, _) => texture.height(),
        }
    }

    pub fn premultiplied(&self) -> bool {
        match &self.source {
            ImageSource::Bitmap(bitmap) => bitmap.info.premultiplied,
            ImageSource::Texture(_, info) => info.premultiplied,
        }
    }

    pub fn get_format(&self) -> ImageFormat {
        match &self.source {
            ImageSource::Bitmap(bitmap) => bitmap.info.format,
            ImageSource::Texture(_, info) => info.format,
        }
    }

    pub fn create_image(
        data: &[u8],
        info: &ImageInfo,
        bytes_per_row: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Image Texture"),
            size: wgpu::Extent3d {
                width: info.width,
                height: info.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: info.format.into(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[info.format.into()],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: info.width,
                height: info.height,
                depth_or_array_layers: 1,
            },
        );

        Self {
            source: ImageSource::Texture(Rc::new(texture), info.clone()),
        }
    }
}

impl From<Rc<Bitmap>> for Image {
    fn from(bitmap: Rc<Bitmap>) -> Self {
        Self {
            source: ImageSource::Bitmap(bitmap),
        }
    }
}

impl From<Bitmap> for Image {
    fn from(bitmap: Bitmap) -> Self {
        Self {
            source: ImageSource::Bitmap(Rc::new(bitmap)),
        }
    }
}

impl From<(Rc<wgpu::Texture>, ImageInfo)> for Image {
    fn from((texture, info): (Rc<wgpu::Texture>, ImageInfo)) -> Self {
        Self {
            source: ImageSource::Texture(texture, info),
        }
    }
}

impl From<(wgpu::Texture, ImageInfo)> for Image {
    fn from((texture, info): (wgpu::Texture, ImageInfo)) -> Self {
        Self {
            source: ImageSource::Texture(Rc::new(texture), info),
        }
    }
}

impl Into<wgpu::TextureFormat> for ImageFormat {
    fn into(self) -> wgpu::TextureFormat {
        match self {
            ImageFormat::BGRA8888 => wgpu::TextureFormat::Bgra8Unorm,
            ImageFormat::RGBA8888 => wgpu::TextureFormat::Rgba8Unorm,
            ImageFormat::BGRX8888 => wgpu::TextureFormat::Bgra8Unorm,
            ImageFormat::RGBX8888 => wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

impl Into<PaintColor> for Image {
    fn into(self) -> PaintColor {
        let width = self.get_width() as f32;
        let height = self.get_height() as f32;

        PaintColor::Image(Rc::new(self), 1.0, Rect::new_xywh(0.0, 0.0, width, height))
    }
}

impl Into<PaintColor> for Rc<Image> {
    fn into(self) -> PaintColor {
        let width = self.get_width() as f32;
        let height = self.get_height() as f32;
        PaintColor::Image(self, 1.0, Rect::new_xywh(0.0, 0.0, width, height))
    }
}

impl Into<PaintColor> for (Rc<Image>, f32) {
    fn into(self) -> PaintColor {
        let width = self.0.get_width() as f32;
        let height = self.0.get_height() as f32;
        PaintColor::Image(self.0, self.1, Rect::new_xywh(0.0, 0.0, width, height))
    }
}

impl Into<PaintColor> for (Rc<Image>, f32, Rect) {
    fn into(self) -> PaintColor {
        PaintColor::Image(self.0, self.1, self.2)
    }
}
