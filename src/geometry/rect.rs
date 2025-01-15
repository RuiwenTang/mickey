use crate::Point;

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
}
