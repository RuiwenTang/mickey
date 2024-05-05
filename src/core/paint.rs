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

/// Controls the Style when rendering geometry
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Style {
    /// Fill the geometry
    #[default]
    Fill,
    /// Stroke the geometry
    Stroke {
        width: f32,
        miter_limit: f32,
        cap: StrokeCap,
        join: StrokeJoin,
    },
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
