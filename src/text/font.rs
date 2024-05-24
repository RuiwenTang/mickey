use std::hash::Hash;

use ab_glyph::{Font as ABFont, FontArc, PxScale, ScaleFont};

/// Describes the style of the font.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontStyle {
    /// The font weight.
    ///
    /// The weight is a number between 100 and 900.
    ///
    /// The default value is 400.
    pub weight: i32,
    pub italic: bool,
    /// The font stretch. The default value is 5.
    pub stretch: i32,
}

impl FontStyle {
    /// The normal font style.
    pub fn normal() -> Self {
        Self {
            weight: 400,
            italic: false,
            stretch: 5,
        }
    }
    /// The bold font style.
    pub fn bold() -> Self {
        Self {
            weight: 700,
            italic: false,
            stretch: 5,
        }
    }
    /// The italic font style.
    pub fn italic() -> Self {
        Self {
            weight: 400,
            italic: true,
            stretch: 5,
        }
    }

    pub fn new(weight: i32, italic: bool, stretch: i32) -> Self {
        Self {
            weight,
            italic,
            stretch,
        }
    }
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::normal()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontDescription {
    /// The font name.
    pub name: String,
    /// The font family.
    pub family: String,
    /// The font style.
    pub style: FontStyle,
}

/// High level abstraction for font.
#[derive(Debug, Clone)]
pub struct Font {
    /// The font description
    pub description: FontDescription,
    /// The font handler
    pub(crate) native_font: FontArc,
}

impl Font {
    pub fn new(desccription: FontDescription, native_font: FontArc) -> Self {
        Self {
            description: desccription,
            native_font,
        }
    }

    pub(crate) fn get_scaled_font(&self, px_size: f32) -> ab_glyph::PxScaleFont<&FontArc> {
        ab_glyph::Font::as_scaled(&self.native_font, PxScale::from(px_size))
    }

    /// Get the ascent of the font. The ascent is the distance from the baseline to the top of the font.
    ///
    /// # Arguments
    ///
    /// * `px_size` - The font size in pixels.
    pub fn get_ascent(&self, px_size: f32) -> f32 {
        self.get_scaled_font(px_size).ascent()
    }

    /// Get the descent of the font. The descent is the distance from the baseline to the bottom of the font.
    ///
    /// # Arguments
    ///
    /// * `px_size` - The font size in pixels.
    pub fn get_descent(&self, px_size: f32) -> f32 {
        self.get_scaled_font(px_size).descent()
    }

    /// Get glyph id from this font.
    pub fn get_glyph_id(&self, c: char) -> u16 {
        self.native_font.glyph_id(c).0
    }
}
