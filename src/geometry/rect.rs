use crate::{Path, Point};
use nalgebra::Matrix4;

/// Rect describes a 2D rectangle.
/// It is a rectangle with the left top point at (left, top) and the right bottom point at (right, bottom).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Rect {
    /// Create a new Rect with the given left, top, right, bottom.
    /// The left and top values must be less than the right and bottom values.
    /// The function not check the input values.
    ///
    /// # Arguments
    /// * `left` - The left value.
    /// * `top` - The top value.
    /// * `right` - The right value.
    /// * `bottom` - The bottom value.
    pub fn new_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Rect {
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Create a new Rect with the given left, top, width, height.
    ///
    /// # Arguments
    /// * `left` - The left value.
    /// * `top` - The top value.
    /// * `width` - The width value.
    /// * `height` - The height value.
    pub fn new_xywh(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    }

    /// Get the left value.
    pub fn left(&self) -> f32 {
        self.left
    }

    /// Get the top value.
    pub fn top(&self) -> f32 {
        self.top
    }

    /// Get the right value.
    pub fn right(&self) -> f32 {
        self.right
    }

    /// Get the bottom value.
    pub fn bottom(&self) -> f32 {
        self.bottom
    }

    /// Get the width value.
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    /// Get the height value.
    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    /// Get the center point.
    pub fn center(&self) -> Point {
        Point::new(
            (self.left + self.right) / 2.0,
            (self.top + self.bottom) / 2.0,
        )
    }

    /// Check if the Rect is empty (width or height is 0).
    pub fn is_empty(&self) -> bool {
        self.width() == 0.0 || self.height() == 0.0
    }

    /// Offset the Rect by the given dx and dy values.
    ///
    /// # Arguments
    /// * `dx` - The x value to offset by.
    /// * `dy` - The y value to offset by.
    ///
    /// # Returns
    /// A new Rect that is the result of offsetting the original Rect by the given dx and dy values.
    pub fn offset(mut self, dx: f32, dy: f32) -> Self {
        self.left += dx;
        self.right += dx;

        self.top += dy;
        self.bottom += dy;

        self
    }

    /// Union the Rect with another Rect.
    /// The resulting Rect will be the smallest Rect that contains both Rects.
    ///
    /// # Arguments
    /// * `other` - The other Rect to union with.
    pub fn union(&mut self, other: &Rect) {
        if other.is_empty() {
            return;
        }

        if self.is_empty() {
            self.left = other.left;
            self.right = other.right;
            self.top = other.top;
            self.bottom = other.bottom;
            return;
        }

        self.left = self.left.min(other.left);
        self.right = self.right.max(other.right);
        self.top = self.top.min(other.top);
        self.bottom = self.bottom.max(other.bottom);
    }
}

/// Convert the Rect into mvp matrix.
/// The matrix is a 4x4 matrix coliumn major for WGPU rendering.
impl Into<[f32; 16]> for Rect {
    fn into(self) -> [f32; 16] {
        let mvp = Matrix4::new_orthographic(
            self.left,
            self.right,
            self.bottom,
            self.top,
            -1000.0,
            1000.0,
        );

        mvp.as_slice()
            .iter()
            .map(|v| *v)
            .collect::<Vec<f32>>()
            .try_into()
            .unwrap()
    }
}

impl Into<Path> for Rect {
    fn into(self) -> Path {
        Path::new()
            .move_to((self.left, self.top))
            .line_to((self.right, self.top))
            .line_to((self.right, self.bottom))
            .line_to((self.left, self.bottom))
            .close_path()
    }
}
