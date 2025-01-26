use std::rc::Rc;

use crate::{Color, Image, LinearGradient, RadialGradient, Rect, Style};

#[derive(Debug, Clone, PartialEq)]
pub enum PaintColor {
    Color(Color),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    Image(Rc<Image>, f32, Rect),
}

/// The paint describes the style and color when drawing a geometry.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Paint {
    pub style: Style,
    pub color: PaintColor,
}

impl Paint {
    pub fn new<T: Into<PaintColor>>(style: Style, color: T) -> Paint {
        Paint {
            style,
            color: color.into(),
        }
    }

    pub fn with_color<T: Into<PaintColor>>(self, color: T) -> Paint {
        Paint {
            color: color.into(),
            ..self
        }
    }

    pub fn set_color<T: Into<PaintColor>>(&mut self, color: T) {
        self.color = color.into();
    }

    /// Change the style of the paint.
    /// The default style is Style::Fill.
    ///
    /// # Arguments
    ///
    /// * `style` - The style [`Style`] of the paint.
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

impl Default for PaintColor {
    fn default() -> Self {
        PaintColor::Color(Color::black())
    }
}

impl PaintColor {
    pub fn set_alpha(&mut self, alpha: f32) {
        match self {
            PaintColor::Color(color) => color.a = alpha,
            PaintColor::LinearGradient(gradient) => gradient.set_alpha(alpha),
            PaintColor::RadialGradient(gradient) => gradient.set_alpha(alpha),
            PaintColor::Image(_, a, _) => *a = alpha,
        }
    }
}
