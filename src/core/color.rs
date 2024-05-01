use bytemuck::{Pod, Zeroable};
use nalgebra::clamp;

/// Unpremultiplied color with RGBA channel
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

fn hsla_hue(h: f32, m1: f32, m2: f32) -> f32 {
    let mut h = h;
    if h < 0.0 {
        h += 1.0;
    }

    if h > 1.0 {
        h -= 1.0;
    }

    if h < 1.0 / 6.0 {
        return m1 + (m2 - m1) * h * 6.0;
    } else if h < 3.0 / 6.0 {
        return m2;
    } else if h < 4.0 / 6.0 {
        return m1 + (m2 - m1) * (2.0 / 3.0 - h) * 6.0;
    } else {
        return m1;
    }
}

impl Color {
    /// Returs color value from rgba component values.
    ///
    /// # Arguments
    ///
    /// * 'r' value of red channel. the value needs between [0.0, 1.0]
    /// * 'g' value of green channel. the value needs between [0.0, 1.0]
    /// * 'b' value of blue channel. the value needs between [0.0, 1.0]
    /// * 'a' value of alpha channel. the value needs between [0.0, 1.0]
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Returs color value from rgba component values.
    ///
    /// # Arguments
    ///
    /// * 'r' amount of red, from no red (0) to full red (255)
    /// * 'g' amount of green, from no green (0) to full green (255)
    /// * 'b' amount of blue, from no blue (0) to full blue (255)
    /// * 'a' amount of alpha, from fully transparent (0) to fully opaque (255)
    pub fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn from_hsla(h: f32, s: f32, l: f32, a: u8) -> Self {
        let mut h = h % 1.0;

        if h < 0.0 {
            h += 1.0;
        }

        let s = clamp(s, 0.0, 1.0);
        let l = clamp(l, 0.0, 1.0);

        let m2 = if l <= 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };

        let m1 = 2.0 * l - m2;

        let r = clamp(hsla_hue(h + 1.0 / 3.0, m1, m2), 0.0, 1.0);
        let g = clamp(hsla_hue(h, m1, m2), 0.0, 1.0);
        let b = clamp(hsla_hue(h - 1.0 / 3.0, m1, m2), 0.0, 1.0);

        Self {
            r,
            g,
            b,
            a: a as f32 / 255.0,
        }
    }

    pub fn transparent() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }

    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    pub fn dark_gray() -> Self {
        Self {
            r: 0.25,
            g: 0.25,
            b: 0.25,
            a: 1.0,
        }
    }

    pub fn light_gray() -> Self {
        Self {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        }
    }

    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    pub fn red() -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    pub fn green() -> Self {
        Self {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }

    pub fn blue() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }

    pub fn yellow() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }

    pub fn cyan() -> Self {
        Self {
            r: 0.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    pub fn magenta() -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }

    pub fn with_alpha(mut self, a: f32) -> Self {
        self.a *= a;

        self
    }
}
