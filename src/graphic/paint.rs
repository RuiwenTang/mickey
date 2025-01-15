use crate::{Color, Style};

/// The paint describes the style and color when drawing a geometry.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Paint {
    pub style: Style,
    pub color: Color,
}

impl Paint {
    pub fn new(style: Style, color: Color) -> Paint {
        Paint { style, color }
    }

    pub fn with_color(self, color: Color) -> Paint {
        Paint { color, ..self }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Change the style of the paint.
    /// The default style is Style::Fill.
    ///
    /// # Arguments
    ///
    /// * `style` - The style of the paint.
    ///
    /// # Returns
    ///
    /// The paint with the new style.
    pub fn with_style<T: Into<Style>>(self, style: T) -> Paint {
        Paint {
            style: style.into(),
            ..self
        }
    }

    /// Set the style of the paint.
    /// The default style is Style::Fill.
    ///
    /// # Arguments
    ///
    /// * `style` - The style of the paint.
    pub fn set_style<T: Into<Style>>(&mut self, style: T) {
        self.style = style.into();
    }
}
