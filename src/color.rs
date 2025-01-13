/// A color with red, green, blue, and alpha components.
/// Each component is a floating-point value between 0.0 and 1.0.
/// All components are unpremultiplied by the alpha value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color with the given red, green, blue, and alpha components.
    ///
    /// # Arguments
    /// * `r` - The red component of the color.
    /// * `g` - The green component of the color.
    /// * `b` - The blue component of the color.
    /// * `a` - The alpha component of the color.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    /// Create a new color with the given red, green, and blue components.
    /// The alpha component is set to 1.0.
    ///
    /// # Arguments
    /// * `r` - The red component of the color.
    /// * `g` - The green component of the color.
    /// * `b` - The blue component of the color.
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    /// Create a new color with the given red, green, blue, and alpha components.
    /// Each component is an unsigned 8-bit integer value between 0 and 255.
    ///
    /// # Arguments
    /// * `r` - The red component of the color.
    /// * `g` - The green component of the color.
    /// * `b` - The blue component of the color.
    /// * `a` - The alpha component of the color.
    pub fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Change the alpha component of the color.
    /// The alpha component is a floating-point value between 0.0 and 1.0.
    ///
    /// # Arguments
    /// * `a` - The new alpha component of the color.
    pub fn with_alpha(self, a: f32) -> Color {
        Color { a, ..self }
    }

    /// Change the alpha component of the color.
    /// The alpha component is an unsigned 8-bit integer value between 0 and 255.
    ///
    /// # Arguments
    /// * `a` - The new alpha component of the color.
    pub fn with_alpha_u8(self, a: u8) -> Color {
        Color {
            a: a as f32 / 255.0,
            ..self
        }
    }

    pub fn white() -> Color {
        Color::from_rgb(1.0, 1.0, 1.0)
    }

    pub fn black() -> Color {
        Color::from_rgb(0.0, 0.0, 0.0)
    }

    pub fn red() -> Color {
        Color::from_rgb(1.0, 0.0, 0.0)
    }

    pub fn green() -> Color {
        Color::from_rgb(0.0, 1.0, 0.0)
    }

    pub fn blue() -> Color {
        Color::from_rgb(0.0, 0.0, 1.0)
    }

    pub fn yellow() -> Color {
        Color::from_rgb(1.0, 1.0, 0.0)
    }

    pub fn cyan() -> Color {
        Color::from_rgb(0.0, 1.0, 1.0)
    }

    pub fn magenta() -> Color {
        Color::from_rgb(1.0, 0.0, 1.0)
    }

    pub fn transparent() -> Color {
        Color::from_rgba_u8(0, 0, 0, 0)
    }

    pub fn lerp(a: Color, b: Color, t: f32) -> Color {
        Color {
            r: a.r + (b.r - a.r) * t,
            g: a.g + (b.g - a.g) * t,
            b: a.b + (b.b - a.b) * t,
            a: a.a + (b.a - a.a) * t,
        }
    }
}

impl Default for Color {
    fn default() -> Color {
        Color::black()
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}
