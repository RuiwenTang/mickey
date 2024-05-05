pub(crate) mod color;
pub(crate) mod geometry;
pub(crate) mod paint;
pub(crate) mod path;
pub(crate) mod picture;
pub(crate) mod state;
pub(crate) mod surface;

use bytemuck::{Pod, Zeroable};
pub use color::Color;
pub use paint::{Paint, StrokeCap, StrokeJoin, Style};
pub use path::{Path, PathFillType};
pub use picture::{Picture, PictureRecorder};
pub use surface::Surface;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn from_highp(x: f64, y: f64) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }

    pub fn from(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
