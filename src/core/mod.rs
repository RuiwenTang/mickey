pub(crate) mod color;
pub(crate) mod paint;
pub(crate) mod path;
pub(crate) mod picture;
pub(crate) mod state;
pub(crate) mod surface;

use bytemuck::{Pod, Zeroable};
pub use color::Color;
pub use paint::Paint;
pub use path::{Path, PathFillType};
pub use picture::{Picture, PictureRecorder};
pub use surface::Surface;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
