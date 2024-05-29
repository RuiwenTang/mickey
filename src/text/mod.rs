use std::rc::Rc;

use ab_glyph::{Glyph, ScaleFont};

pub(crate) mod font;
pub(crate) mod glyph_atlas;

pub use font::{Font, FontDescription, FontStyle};

use crate::core::{Bitmap, ImageFormat, ImageInfo};

pub struct TextRun {
    pub(crate) glyphs: Vec<Glyph>,
    pub px_size: f32,
    pub font: Rc<Font>,
}

impl TextRun {
    pub fn new(chars: Vec<char>, font: Rc<Font>, px_size: f32) -> Self {
        let fs = font.get_scaled_font(px_size);

        let advance = fs.ascent();

        let mut glyphs: Vec<Glyph> = Vec::new();

        let mut prev_gryph: Option<Glyph> = None;
        let mut x = 0.0;
        for c in chars.iter() {
            let mut g = fs.scaled_glyph(*c);

            if let Some(pg) = prev_gryph.take() {
                x += fs.kern(pg.id, g.id);
            }
            g.position.x = x;
            g.position.y = advance;

            if !c.is_whitespace() {
                glyphs.push(g.clone());
            }
            prev_gryph = Some(g.clone());
            x += fs.h_advance(g.id);
        }

        Self {
            glyphs,
            px_size,
            font,
        }
    }

    /// Get all glyphs in this run.
    pub fn get_glyphs(&self) -> Vec<u16> {
        self.glyphs.iter().map(|g| g.id.0).collect()
    }

    /// Get all glyphs position at x-axis in this run.
    pub fn get_glyphs_position(&self) -> Vec<f32> {
        self.glyphs.iter().map(|g| g.position.x).collect()
    }
}

pub struct TextBlob {
    pub runs: Vec<TextRun>,

    pub width: f32,
    pub height: f32,

    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
}

impl TextBlob {
    pub fn new(runs: Vec<TextRun>) -> Self {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        let mut ascent: f32 = 0.0;
        let mut descent: f32 = 0.0;
        let mut line_gap: f32 = 0.0;

        for run in runs.iter() {
            let fs = run.font.get_scaled_font(run.px_size);
            let run_width =
                run.glyphs.last().unwrap().position.x + fs.h_advance(run.glyphs.last().unwrap().id);
            width += run_width;

            let run_height = fs.ascent() - fs.descent() + fs.line_gap();
            height = height.max(run_height);

            ascent = ascent.max(fs.ascent());
            descent = descent.min(fs.descent());
            line_gap = fs.line_gap();
        }

        Self {
            runs,
            width,
            height,
            ascent,
            descent,
            line_gap,
        }
    }

    /// Raster this blob to bitmap. Only for debug.
    pub fn raster_to_image(&self) -> Bitmap {
        let width = self.width.ceil() as u32;
        let height = self.height.ceil() as u32;

        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize((width * height * 4) as usize, 0);

        for run in self.runs.iter() {
            let scaled_font = run.font.get_scaled_font(run.px_size);
            for glyph in run.glyphs.iter() {
                let mut g = glyph.clone();
                g.position.y = scaled_font.ascent();
                if let Some(outlined) = scaled_font.outline_glyph(g) {
                    let bounds = outlined.px_bounds();

                    outlined.draw(|x, y, v| {
                        let x = x + bounds.min.x as u32;
                        let y = y + bounds.min.y as u32;
                        let index = y * width * 4 + x * 4;
                        buffer[index as usize + 3] = (v * 255.0) as u8;
                    });
                }
            }
        }

        return Bitmap::new(
            ImageInfo {
                width,
                height,
                format: ImageFormat::RGBA8888,
                premultiplied: false,
            },
            buffer,
            None,
        );
    }
}

pub struct TextBlobBuilder {
    font: Rc<Font>,
    px_size: f32,

    fallback_font: Vec<Rc<Font>>,
}

impl TextBlobBuilder {
    pub fn new(font: Rc<Font>, px_size: f32) -> Self {
        Self {
            font,
            px_size,
            fallback_font: Vec::new(),
        }
    }

    pub fn with_fallback_font(mut self, font: Rc<Font>) -> Self {
        self.fallback_font.push(font);
        self
    }

    pub fn build(&self, text: &str) -> Rc<TextBlob> {
        let mut runs: Vec<TextRun> = Vec::new();

        let chars = text.chars().collect::<Vec<char>>();

        let mut run_chars: Vec<char> = Vec::new();

        let mut curr_font = self.font.clone();
        for c in chars.iter() {
            if curr_font.get_glyph_id(*c) == 0 {
                match self.fallback_font(*c) {
                    Some(f) => {
                        if !run_chars.is_empty() {
                            runs.push(TextRun::new(
                                run_chars.clone(),
                                curr_font.clone(),
                                self.px_size,
                            ));
                            run_chars.clear();
                        }
                        curr_font = f;
                    }
                    None => {}
                }
            }
            run_chars.push(*c);
        }

        if !run_chars.is_empty() {
            runs.push(TextRun::new(run_chars, curr_font, self.px_size));
        }

        Rc::new(TextBlob::new(runs))
    }

    fn fallback_font(&self, c: char) -> Option<Rc<Font>> {
        for font in self.fallback_font.iter() {
            if font.get_glyph_id(c) != 0 {
                return Some(font.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use ab_glyph::FontArc;

    use self::font::FontDescription;

    use super::*;

    #[test]
    fn test_font() {
        let font = Font::new(
            FontDescription {
                name: "0xProtoNerdFont-Regular".to_string(),
                family: "0xProtoNerdFont".to_string(),
                style: FontStyle::normal(),
            },
            FontArc::try_from_slice(include_bytes!(
                "../../examples/assets/0xProto/0xProtoNerdFont-Regular.ttf"
            ))
            .expect("Failed to load font"),
        );

        assert!(font.get_glyph_id('a') != 0);
    }

    #[test]
    fn test_text_blob_builder() {
        let font = Font::new(
            FontDescription {
                name: "0xProtoNerdFont-Regular".to_string(),
                family: "0xProtoNerdFont".to_string(),
                style: FontStyle::normal(),
            },
            FontArc::try_from_slice(include_bytes!(
                "../../examples/assets/0xProto/0xProtoNerdFont-Regular.ttf"
            ))
            .expect("Failed to load font"),
        );

        let builder = TextBlobBuilder::new(Rc::new(font), 10.0);

        let blob = builder.build("hello world");

        assert!(!blob.runs.is_empty());
        assert_eq!(blob.runs.len(), 1);
        assert_eq!(blob.runs[0].glyphs.len(), 10);
    }
}
