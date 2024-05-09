pub(crate) mod color;
pub(crate) mod geometry;
pub(crate) mod paint;
pub(crate) mod path;
pub(crate) mod picture;
pub(crate) mod state;
pub(crate) mod surface;

use bytemuck::{Pod, Zeroable};
pub use color::Color;
pub use paint::{Paint, Stroke, StrokeCap, StrokeJoin, Style};
pub use path::{Path, PathDirection, PathFillType};
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

    pub fn offset(&mut self, dx: f32, dy: f32) {
        self.left += dx;
        self.top += dy;
        self.right += dx;
        self.bottom += dy;
    }

    pub fn is_finite(&self) -> bool {
        let mut accum = 0.0;
        accum *= self.left;
        accum *= self.top;
        accum *= self.right;
        accum *= self.bottom;

        return accum.is_finite();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RRect {
    pub(crate) rect: Rect,
    pub(crate) radii: [Point; 4],
}

impl RRect {
    /// Create a special RRect as oval. The radius of oval is half of the width and height of the rect.
    ///
    /// # Arguments
    ///
    /// * `rect` - The bounds of the Oval.
    pub fn new_oval(rect: Rect) -> Self {
        let rx = rect.width() / 2.0;
        let ry = rect.height() / 2.0;

        Self {
            rect,
            radii: [
                Point { x: rx, y: ry },
                Point { x: rx, y: ry },
                Point { x: rx, y: ry },
                Point { x: rx, y: ry },
            ],
        }
    }

    /// Creates a new RRect from a rectangle and corner radii of x-axis and y-axis.
    ///
    /// # Arguments
    ///
    /// * `rect` - The bounds of the RRect.
    /// * `rx` - The radius of the RRect on the x-axis.
    /// * `ry` - The radius of the RRect on the y-axis.
    pub fn from_rect_xy(rect: Rect, rx: f32, ry: f32) -> Self {
        let mut x = rx;
        let mut y = ry;

        if rect.width() < rx + rx || rect.height() < ry + ry {
            let sx = rect.width() / (rx + rx);
            let sy = rect.height() / (ry + ry);

            let s = sx.min(sy);

            x *= s;
            y *= s;
        }

        Self {
            rect,
            radii: [
                Point { x, y },
                Point { x, y },
                Point { x, y },
                Point { x, y },
            ],
        }
    }

    /// Creates a new RRect from a rectangle and corner radii.
    ///
    /// # Arguments
    ///
    /// * `rect` - The bounds of the RRect.
    /// * `radii` - The corner radii of the RRect.
    pub fn from_rect_radii(rect: Rect, radii: [Point; 4]) -> Self {
        Self { rect, radii }
    }

    /// Returns the width of the rectangle.
    /// This dose not check if RRect is sorted.
    /// Result may be negative.
    pub fn width(&self) -> f32 {
        self.rect.width()
    }

    /// Returns the height of the rectangle.
    /// This dose not check if RRect is sorted.
    /// Result may be negative.
    pub fn height(&self) -> f32 {
        self.rect.height()
    }

    pub fn center(&self) -> Point {
        self.rect.center()
    }

    pub fn bounds(&self) -> Rect {
        self.rect
    }

    pub fn is_empty(&self) -> bool {
        self.rect.is_empty()
    }

    pub fn offset(&mut self, dx: f32, dy: f32) {
        self.rect.offset(dx, dy);
    }

    pub fn is_rect(&self) -> bool {
        self.radii[0].x == 0.0
            && self.radii[0].x == self.radii[1].x
            && self.radii[0].x == self.radii[2].x
            && self.radii[0].x == self.radii[3].x
            && self.radii[0].y == self.radii[1].y
            && self.radii[0].y == self.radii[2].y
            && self.radii[0].y == self.radii[3].y
    }

    pub fn is_oval(&self) -> bool {
        self.radii[0].x == self.width() * 0.5
            && self.radii[0].y == self.height() * 0.5
            && self.radii[0].x == self.radii[1].x
            && self.radii[0].x == self.radii[2].x
            && self.radii[0].x == self.radii[3].x
            && self.radii[0].y == self.radii[1].y
            && self.radii[0].y == self.radii[2].y
            && self.radii[0].y == self.radii[3].y
    }
}
