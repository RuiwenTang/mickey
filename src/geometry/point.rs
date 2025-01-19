use std::ops::{Add, Div, Mul, Sub};

use bytemuck::{Pod, Zeroable};

/// A point in 2D space with an x and y coordinate.
/// The coordinates are floating-point values.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Create a new point with the given x and y coordinates.
    ///
    /// # Arguments
    /// * `x` - The x coordinate of the point.
    /// * `y` - The y coordinate of the point.
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    /// Create a new point with the given x and y coordinates.
    /// The coordinates are double precision floating-point values.
    ///
    /// # Arguments
    /// * `x` - The x coordinate of the point.
    /// * `y` - The y coordinate of the point.
    pub fn new_highp(x: f64, y: f64) -> Point {
        Point {
            x: x as f32,
            y: y as f32,
        }
    }

    /// Check if the point is a NaN value.
    ///
    /// # Returns
    /// `true` if one of the coordinates is a NaN value, `false` otherwise.
    pub fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }
}

impl Add<Point> for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<f32> for Point {
    type Output = Point;
    fn add(self, other: f32) -> Point {
        Point {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl Sub<Point> for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub<f32> for Point {
    type Output = Point;
    fn sub(self, other: f32) -> Point {
        Point {
            x: self.x - other,
            y: self.y - other,
        }
    }
}

impl Mul<Point> for Point {
    type Output = Point;
    fn mul(self, other: Point) -> Point {
        Point {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, other: f32) -> Point {
        Point {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<Point> for Point {
    type Output = Point;
    fn div(self, other: Point) -> Point {
        Point {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl Div<f32> for Point {
    type Output = Point;
    fn div(self, other: f32) -> Point {
        Point {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl From<[f32; 2]> for Point {
    fn from([x, y]: [f32; 2]) -> Self {
        Point { x, y }
    }
}

impl From<[f64; 2]> for Point {
    fn from([x, y]: [f64; 2]) -> Self {
        Point {
            x: x as f32,
            y: y as f32,
        }
    }
}

impl Into<Point> for (f32, f32) {
    fn into(self) -> Point {
        Point {
            x: self.0,
            y: self.1,
        }
    }
}

impl Into<Point> for (f64, f64) {
    fn into(self) -> Point {
        Point {
            x: self.0 as f32,
            y: self.1 as f32,
        }
    }
}
