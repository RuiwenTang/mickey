use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    RGBA8888,
    BGRA8888,
    RGBX8888,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub premultiplied: bool,
}

/// Represent a immutable image. Since we only use gpu to do low level rendering.
#[derive(Debug, Clone)]
pub struct Bitmap {
    pub data: Vec<u8>,
    pub info: ImageInfo,
    pub bytes_per_row: u32,
}

impl Bitmap {
    pub fn new(info: ImageInfo, data: Vec<u8>, bytes_per_row: Option<u32>) -> Self {
        let bytes_per_row = bytes_per_row.unwrap_or(info.width as u32 * 4);
        Self {
            data,
            info,
            bytes_per_row,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ImageSource {
    Bitmap(Rc<Bitmap>),
    Texture(Rc<wgpu::Texture>, ImageInfo),
}

/// A wrapper for a Bitamp or a wgpu::Texture.
#[derive(Debug, Clone)]
pub struct Image {
    pub(crate) source: ImageSource,
}

impl Image {
    /// Construct a new `Image` from a `Bitmap`.
    /// Since we only use gpu to do low level rendering.
    /// This bitmap will create a texture and upload the data to gpu every time.
    pub fn from_bitmap(bitmap: Rc<Bitmap>) -> Self {
        Self {
            source: ImageSource::Bitmap(bitmap),
        }
    }

    /// Construct a new `Image` from a `wgpu::Texture`.
    pub fn from_texture(texture: Rc<wgpu::Texture>, info: ImageInfo) -> Self {
        Self {
            source: ImageSource::Texture(texture, info),
        }
    }

    pub fn width(&self) -> u32 {
        match &self.source {
            ImageSource::Bitmap(bitmap) => bitmap.info.width,
            ImageSource::Texture(_, info) => info.width,
        }
    }

    pub fn height(&self) -> u32 {
        match &self.source {
            ImageSource::Bitmap(bitmap) => bitmap.info.height,
            ImageSource::Texture(_, info) => info.height,
        }
    }
}
