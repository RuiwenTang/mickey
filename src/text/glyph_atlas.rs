use std::{hash::Hash, rc::Rc};

use ab_glyph::{Font as ABFont, Glyph, ScaleFont};

use crate::{core::Rect, gpu::atlas::AtlasTexture};

use super::{Font, FontDescription};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GlyphAtlasKey {
    font: FontDescription,
    id: u16,
    px_size: f32,
}

impl Eq for GlyphAtlasKey {}

impl Hash for GlyphAtlasKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.font.hash(state);
        self.id.hash(state);
        let upx = (self.px_size * 1000.0).ceil() as u32;
        upx.hash(state);
    }
}

const TEXTURE_SIZE: u32 = 2048;
const REGION_PADDING: u32 = 1;

pub(crate) struct GlyphAtlasValue {
    pub(crate) rect: Rect,
    pub(crate) texture: Rc<wgpu::Texture>,
}

pub(crate) struct GlyphAtlasManager {
    format: wgpu::TextureFormat,
    index: usize,
    textures: Vec<AtlasTexture<GlyphAtlasKey>>,
}

impl GlyphAtlasManager {
    pub(crate) fn new(format: wgpu::TextureFormat, device: &wgpu::Device) -> Self {
        Self {
            format,
            index: 0,
            textures: vec![AtlasTexture::<GlyphAtlasKey>::new(
                TEXTURE_SIZE,
                TEXTURE_SIZE,
                format,
                device,
            )],
        }
    }

    pub(crate) fn query_atlas_region(
        &self,
        font: &Font,
        glyph: &Glyph,
        px_size: f32,
    ) -> Option<GlyphAtlasValue> {
        let key = &GlyphAtlasKey {
            font: font.description.clone(),
            id: glyph.id.0,
            px_size,
        };

        for i in 0..(self.index + 1) {
            let region = self.textures[i].query_region(key);

            match region {
                None => continue,
                Some((l, t, w, h)) => {
                    let (lf, tf) = self.textures[self.index].pos_to_uv(l, t);
                    let (rf, bf) = self.textures[self.index].pos_to_uv(l + w, t + h);

                    return Some(GlyphAtlasValue {
                        rect: Rect::from_ltrb(lf, tf, rf, bf),
                        texture: self.textures[self.index].get_texture(),
                    });
                }
            }
        }

        None
    }

    pub(crate) fn alloc_atlas_region(
        &mut self,
        font: &Font,
        glyph: &Glyph,
        px_size: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<GlyphAtlasValue> {
        let fs = font.native_font.as_scaled(px_size);

        let key = &GlyphAtlasKey {
            font: font.description.clone(),
            id: glyph.id.0,
            px_size: px_size,
        };

        let outline = fs
            .outline_glyph(glyph.clone())
            .expect("Font and glyph not match!!");

        let bounds = outline.px_bounds();
        let width = bounds.width().ceil() as u32 + REGION_PADDING * 2;
        let height = bounds.height().ceil() as u32 + REGION_PADDING * 2;

        let mut region = self.textures[self.index].alloc_region(key, width, height);

        if region.is_none() {
            self.textures.push(AtlasTexture::new(
                TEXTURE_SIZE,
                TEXTURE_SIZE,
                self.format,
                device,
            ));

            self.index += 1;

            region = self.textures[self.index].alloc_region(key, width, height);
        }

        if region.is_none() {
            return None;
        }

        let (x, y, w, h) = region.unwrap();

        {
            let mut data: Vec<u8> = Vec::new();
            data.resize((w * h) as usize, 0);

            outline.draw(|x, y, v| {
                let offset = (y + REGION_PADDING) * w + x + REGION_PADDING;
                data[offset as usize] = (v * 255.0) as u8;
            });

            self.textures[self.index].upload(data.as_slice(), x, y, w, h, queue);
        }

        let (lf, tf) = self.textures[self.index].pos_to_uv(x, y);
        let (rf, bf) = self.textures[self.index].pos_to_uv(x + w, y + h);
        return Some(GlyphAtlasValue {
            rect: Rect::from_ltrb(lf, tf, rf, bf),
            texture: self.textures[self.index].get_texture(),
        });
    }

    pub(crate) fn get_total_memory(&self) -> usize {
        self.textures.len() * TEXTURE_SIZE as usize * TEXTURE_SIZE as usize
    }

    pub(crate) fn get_used_memory(&self) -> usize {
        let used_area: f32 = self.textures.iter().map(|t| t.get_used_area()).sum();

        used_area as usize
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::text::FontStyle;

    use super::*;

    #[test]
    fn test_glyph_atlas_key() {
        let fd = FontDescription {
            name: "0xProtoNerdFont-Regular".to_string(),
            family: "0xProtoNerdFont".to_string(),
            style: FontStyle::normal(),
        };

        let key1 = GlyphAtlasKey {
            font: fd.clone(),
            id: 1,
            px_size: 15.0,
        };

        assert_eq!(
            &key1,
            &GlyphAtlasKey {
                font: fd.clone(),
                id: 1,
                px_size: 15.0,
            }
        );

        assert_ne!(
            &key1,
            &GlyphAtlasKey {
                font: fd.clone(),
                id: 1,
                px_size: 15.2,
            }
        )
    }

    #[test]
    fn test_glyph_map() {
        let mut map: HashMap<GlyphAtlasKey, u32> = HashMap::new();

        let fd = FontDescription {
            name: "0xProtoNerdFont-Regular".to_string(),
            family: "0xProtoNerdFont".to_string(),
            style: FontStyle::normal(),
        };

        map.insert(
            GlyphAtlasKey {
                font: fd.clone(),
                id: 1,
                px_size: 15.0,
            },
            1,
        );

        map.insert(
            GlyphAtlasKey {
                font: fd.clone(),
                id: 1,
                px_size: 16.0,
            },
            2,
        );

        map.insert(
            GlyphAtlasKey {
                font: fd.clone(),
                id: 2,
                px_size: 15.0,
            },
            3,
        );

        let v1 = map.get(&GlyphAtlasKey {
            font: fd.clone(),
            id: 1,
            px_size: 15.0,
        });

        assert!(v1.is_some());
        assert_eq!(v1.unwrap(), &1);

        let v2 = map.get(&GlyphAtlasKey {
            font: fd.clone(),
            id: 1,
            px_size: 16.0,
        });

        assert!(v2.is_some());
        assert_eq!(v2, Some(&2));

        let v3 = map.get(&GlyphAtlasKey {
            font: fd.clone(),
            id: 2,
            px_size: 15.0,
        });

        assert!(v3.is_some());
        assert_eq!(v3, Some(&3));

        let v4 = map.get(&GlyphAtlasKey {
            font: fd.clone(),
            id: 3,
            px_size: 15.0,
        });

        assert!(v4.is_none());
    }
}
