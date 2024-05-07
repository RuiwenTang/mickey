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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// The left coordinate of the rectangle. If sorted.
    pub left: f32,
    /// The top coordinate of the rectangle. If sorted.
    pub top: f32,
    /// The right coordinate of the rectangle. If sorted.
    pub right: f32,
    /// The bottom coordinate of the rectangle. If sorted.
    pub bottom: f32,
}

impl Rect {
    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    }

    /// Returns the width of the rectangle.
    /// This dose not check if Rect is sorted.
    /// Result may be negative.
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    /// Returns the height of the rectangle.
    /// This dose not check if Rect is sorted.
    /// Result may be negative.
    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    pub fn center(&self) -> Point {
        Point {
            x: self.left + self.width() / 2.0,
            y: self.top + self.height() / 2.0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    pub fn is_sorted(&self) -> bool {
        self.left <= self.right && self.top <= self.bottom
    }

    pub fn sort(&mut self) {
        if self.left > self.right {
            std::mem::swap(&mut self.left, &mut self.right);
        }

        if self.top > self.bottom {
            std::mem::swap(&mut self.top, &mut self.bottom);
        }
    }
}
