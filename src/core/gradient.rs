use nalgebra::Matrix4;

use super::{paint::ColorType, Color, Point, TileMode};

/// A gradient with linear direction between two points.
#[derive(Debug, Clone)]
pub struct LinearGradient {
    pub matrix: Matrix4<f32>,
    /// The colors to be distributed between the two points.
    pub colors: Vec<Color>,
    /// The position of each color in the gradient. Can be empty or must have same length as `colors`.
    /// # Notes:
    /// The stops must be in ascending order.
    pub stops: Vec<f32>,
    /// Start point of the gradient.
    pub p1: Point,
    /// End point of the gradient.
    pub p2: Point,
    /// Defines how to repeat, fold or imit colors outside of the typically defined range of the source of the colors (such as the bounds of an image or the defining geometry of a gradient).
    pub tile_mode: TileMode,
}

impl LinearGradient {
    /// Create a new linear gradient with two points.
    ///
    /// # Arguments
    ///
    /// * `p1` - The start point of the gradient.
    /// * `p2` - The end point of the gradient.
    pub fn new(p1: Point, p2: Point) -> Self {
        Self {
            matrix: Matrix4::identity(),
            colors: Vec::new(),
            stops: Vec::new(),
            p1,
            p2,
            tile_mode: Default::default(),
        }
    }

    /// Add a color to the gradient.
    pub fn add_color(mut self, color: Color) -> Self {
        self.colors.push(color);
        self
    }

    /// Replace the colors of the gradient. The stops will be cleared.
    ///
    /// # Arguments
    ///
    /// * `colors` - The colors to be distributed between the two points.
    pub fn with_colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self.stops.clear();
        self
    }

    /// Replace the colors and stops of the gradient.
    ///
    /// # Arguments
    ///
    /// * `colors` - The colors to be distributed between the two points.
    /// * `stops` - The position of each color in the gradient. Can be empty or must have same length as `colors`.
    pub fn with_colors_stops(mut self, colors: Vec<Color>, stops: Vec<f32>) -> Self {
        self.colors = colors;
        self.stops = stops;
        self
    }

    /// Replace the transform matrix of the gradient.
    /// The transform matrix is used to transform the gradient to another coordinate space.
    pub fn with_matrix(mut self, matrix: Matrix4<f32>) -> Self {
        self.matrix = matrix;
        self
    }

    /// Replace the tile mode of the gradient.
    pub fn with_tile_mode(mut self, tile_mode: TileMode) -> Self {
        self.tile_mode = tile_mode;
        self
    }
}

impl Into<ColorType> for LinearGradient {
    fn into(self) -> ColorType {
        ColorType::LinearGradient(self)
    }
}

/// A gradient with a given center point and radius.
#[derive(Debug, Clone)]
pub struct RadialGradient {
    pub matrix: Matrix4<f32>,
    /// The colors to be distributed between the two points.
    pub colors: Vec<Color>,
    /// The position of each color in the gradient. Can be empty or must have same length as `colors
    /// # Notes:
    /// The stops must be in ascending order.
    pub stops: Vec<f32>,
    /// Center point of the gradient.
    pub center: Point,
    /// Radius of the gradient.
    pub radius: f32,
    /// Defines how to repeat, fold or imit colors outside of the typically defined range of the source of the colors (such as the bounds of an image or the defining geometry of a gradient).
    pub tile_mode: TileMode,
}

impl RadialGradient {
    /// Create a new radial gradient with a center point and radius.
    ///
    /// # Arguments
    ///
    /// * `center` - The center point of the gradient.
    /// * `radius` - The radius of the gradient.
    pub fn new(center: Point, radius: f32) -> Self {
        Self {
            matrix: Matrix4::identity(),
            colors: Vec::new(),
            stops: Vec::new(),
            center,
            radius,
            tile_mode: Default::default(),
        }
    }

    /// Add a color to the gradient.
    pub fn add_color(mut self, color: Color) -> Self {
        self.colors.push(color);
        self
    }

    /// Replace the colors of the gradient. The stops will be cleared.
    ///
    /// # Arguments
    ///
    /// * `colors` - The colors to be distributed between the two points.
    pub fn with_colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self.stops.clear();
        self
    }

    /// Replace the colors and stops of the gradient.
    ///
    /// # Arguments
    ///
    /// * `colors` - The colors to be distributed between the two points.
    /// * `stops` - The position of each color in the gradient. Can be empty or must have same length as `colors`.
    pub fn with_colors_stops(mut self, colors: Vec<Color>, stops: Vec<f32>) -> Self {
        self.colors = colors;
        self.stops = stops;
        self
    }

    /// Replace the transform matrix of the gradient.
    /// The transform matrix is used to transform the gradient to another coordinate space.
    pub fn with_matrix(mut self, matrix: Matrix4<f32>) -> Self {
        self.matrix = matrix;
        self
    }

    /// Replace the tile mode of the gradient.
    pub fn with_tile_mode(mut self, tile_mode: TileMode) -> Self {
        self.tile_mode = tile_mode;
        self
    }
}

impl Into<ColorType> for RadialGradient {
    fn into(self) -> ColorType {
        ColorType::RadialGradient(self)
    }
}
