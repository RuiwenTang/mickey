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

pub struct Degree<T: Into<f32>>(pub T);
pub struct Radian<T: Into<f32>>(pub T);

pub trait Angle {
    fn radians(self) -> f32;
}

impl<T: Into<f32>> Angle for Degree<T> {
    fn radians(self) -> f32 {
        let value = self.0.into();

        value * PI / 180.0
    }
}

impl<T: Into<f32>> Angle for Radian<T> {
    fn radians(self) -> f32 {
        self.0.into()
    }
}

impl Angle for f32 {
    fn radians(self) -> f32 {
        self * PI / 180.0
    }
}

pub trait IntoAngle<T: Into<f32>> {
    fn degree(self) -> Degree<T>;
    fn radian(self) -> Radian<T>;
}

impl IntoAngle<f32> for f32 {
    fn degree(self) -> Degree<f32> {
        Degree(self)
    }

    fn radian(self) -> Radian<f32> {
        Radian(self)
    }
}
