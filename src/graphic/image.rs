use std::rc::Rc;

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

    pub fn get_format(&self) -> ImageFormat {
        match &self.source {
            ImageSource::Bitmap(bitmap) => bitmap.info.format,
            ImageSource::Texture(_, info) => info.format,
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
