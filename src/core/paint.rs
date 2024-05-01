use super::Color;

/// Paint controls options applied when drawing.
#[derive(Debug, Clone, Copy)]
pub struct Paint {
    /// unpremultiplied color used when stroking or filling.
    /// default value is black
    pub color: Color,
}

impl Paint {
    pub fn new() -> Self {
        Self {
            color: Color::black(),
        }
    }
}
