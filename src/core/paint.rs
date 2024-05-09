use super::Color;

/// Cap draws at the beginning and end of an open path contour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeCap {
    /// no stroke extension
    #[default]
    Butt,
    /// adds circle
    Round,
    /// adds square
    Square,
}

/// Specifies how corners are drawn when a shape is stroked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeJoin {
    /// draw sharp corners. extends to the miter limit
    #[default]
    Miter,
    /// draw a circle with a radius equal to the stroke width on top of the corner
    Round,
    /// minimally connect the thick strokes.
    Bevel,
}

/// Specifies the style of the stroke.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    /// width of the stroke.
    /// default value is 1.0
    pub width: f32,
    /// limit for miter joins.
    /// default value is 10.0
    pub miter_limit: f32,
    /// cap style for the stroke
    /// default value is StrokeCap::Butt
    pub cap: StrokeCap,
    /// join style for the stroke
    /// default value is StrokeJoin::Miter
    pub join: StrokeJoin,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            miter_limit: 4.0,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
        }
    }
}

impl Stroke {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_miter_limit(mut self, miter_limit: f32) -> Self {
        self.miter_limit = miter_limit;
        self
    }

    pub fn with_cap(mut self, cap: StrokeCap) -> Self {
        self.cap = cap;
        self
    }

    pub fn with_join(mut self, join: StrokeJoin) -> Self {
        self.join = join;
        self
    }
}

/// Controls the Style when rendering geometry
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Style {
    /// Fill the geometry
    #[default]
    Fill,
    /// Stroke the geometry
    Stroke(Stroke),
}

impl Into<Style> for Stroke {
    fn into(self) -> Style {
        Style::Stroke(self)
    }
}

/// Paint controls options applied when drawing.
#[derive(Debug, Clone, Copy)]
pub struct Paint {
    /// unpremultiplied color used when stroking or filling.
    /// default value is black
    pub color: Color,
    /// style when rendering geometry
    /// default value is Style::Fill
    pub style: Style,
}

impl Paint {
    pub fn new() -> Self {
        Self {
            color: Color::black(),
            style: Style::Fill,
        }
    }
}
