use nalgebra::Matrix4;

use super::{paint::ColorType, Color, Point};

#[derive(Debug, Clone)]
pub struct LinearGradient {
    pub matrix: Matrix4<f32>,
    pub colors: Vec<Color>,
    pub stops: Vec<f32>,
    pub p1: Point,
    pub p2: Point,
}

impl LinearGradient {
    pub fn new(p1: Point, p2: Point) -> Self {
        Self {
            matrix: Matrix4::identity(),
            colors: Vec::new(),
            stops: Vec::new(),
            p1,
            p2,
        }
    }

    pub fn add_color(mut self, color: Color) -> Self {
        self.colors.push(color);
        self
    }

    pub fn with_colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self
    }

    pub fn with_colors_stops(mut self, colors: Vec<Color>, stops: Vec<f32>) -> Self {
        self.colors = colors;
        self.stops = stops;
        self
    }

    pub fn with_matrix(mut self, matrix: Matrix4<f32>) -> Self {
        self.matrix = matrix;
        self
    }
}

impl Into<ColorType> for LinearGradient {
    fn into(self) -> ColorType {
        ColorType::LinearGradient(self)
    }
}
