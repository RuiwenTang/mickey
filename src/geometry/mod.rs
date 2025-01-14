mod matrix;
mod point;

use std::f32::consts::PI;

pub use matrix::*;
pub use point::*;

pub struct Degrees;
pub struct Radians;

pub trait Angle {
    fn to_radians(value: f32) -> f32;
}

impl Angle for Degrees {
    fn to_radians(value: f32) -> f32 {
        value * PI / 180.0
    }
}

impl Angle for Radians {
    fn to_radians(value: f32) -> f32 {
        value
    }
}
