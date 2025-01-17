mod curve;
mod matrix;
mod path;
mod point;
mod rect;

use std::f32::consts::PI;

pub use matrix::*;
pub use path::*;
pub use point::*;
pub use rect::*;

pub(crate) use curve::*;

pub struct Degree(pub f32);
pub struct Radian(pub f32);

pub trait Angle {
    fn radians(self) -> f32;
}

impl Angle for Degree {
    fn radians(self) -> f32 {
        self.0 * PI / 180.0
    }
}

impl Angle for Radian {
    fn radians(self) -> f32 {
        self.0
    }
}

impl Angle for f32 {
    fn radians(self) -> f32 {
        self * PI / 180.0
    }
}
