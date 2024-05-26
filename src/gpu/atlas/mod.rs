mod allocator;

use std::{collections::HashMap, hash::Hash, rc::Rc};

use allocator::AtlasAllocator;

pub(crate) struct AtlasTexture<KEY: Hash + PartialEq + Eq + Clone> {
    width: u32,
    height: u32,

    allocator: AtlasAllocator,
    format: wgpu::TextureFormat,
    texture: Rc<wgpu::Texture>,

    regions: HashMap<KEY, (u32, u32, u32, u32)>,
}

impl<KEY> AtlasTexture<KEY>
where
    KEY: Hash + PartialEq + Eq + Clone,
{
    pub(crate) fn new(
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[format],
        });

        Self {
            width,
            height,
            allocator: AtlasAllocator::new(width, height),
            format,
            texture: Rc::new(texture),
            regions: HashMap::new(),
        }
    }

    pub(crate) fn query_region(&self, key: &KEY) -> Option<(u32, u32, u32, u32)> {
        self.regions.get(key).copied()
    }

    pub(crate) fn alloc_region(
        &mut self,
        key: &KEY,
        width: u32,
        height: u32,
    ) -> Option<(u32, u32, u32, u32)> {
        let region = self.allocate(width, height);

        match &region {
            None => {}
            Some(rect) => {
                self.regions.insert(key.clone(), rect.clone());
            }
        }

        return region;
    }

    fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
        let rect = self.allocator.allocate(width, height);

        if let Some(rect) = rect {
            return Some((rect.x, rect.y, rect.width, rect.height));
        } else {
            return None;
        }
    }

    pub(crate) fn pos_to_uv(&self, x: u32, y: u32) -> (f32, f32) {
        let x = x as f32 / self.width as f32;
        let y = y as f32 / self.height as f32;

        (x, y)
    }

    pub(crate) fn get_use_rate(&self) -> f32 {
        self.allocator.get_use_rate()
    }

    pub(crate) fn get_texture(&self) -> Rc<wgpu::Texture> {
        self.texture.clone()
    }

    pub(crate) fn upload(
        &self,
        data: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        queue: &wgpu::Queue,
    ) {
        let bpp = self
            .format
            .target_pixel_byte_cost()
            .expect("invalid format");
        queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: self.texture.as_ref(),
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: Default::default(),
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bpp * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}
