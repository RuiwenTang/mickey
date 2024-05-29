use std::{ops::Range, rc::Rc};

use nalgebra::{Matrix4, Vector4};

use crate::{
    core::{
        image::{Bitmap, ImageFormat},
        ImageInfo,
    },
    gpu::{buffer::StageBuffer, pipeline::Pipeline, GPUContext},
    render::Fragment,
};

use super::{TransformGroup, TEXTURE_PIPELINE_NAME};

trait TextureProvider {
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);

    fn get_texture(&self) -> Option<Rc<wgpu::Texture>>;

    fn get_width(&self) -> u32;

    fn get_height(&self) -> u32;

    fn get_format(&self) -> ImageFormat;

    fn is_premutied(&self) -> bool;
}

struct BitmapTextureProvider {
    bitmap: Rc<Bitmap>,
    texture: Option<Rc<wgpu::Texture>>,
}

impl BitmapTextureProvider {
    fn new(bitmap: Rc<Bitmap>) -> Self {
        Self {
            bitmap,
            texture: None,
        }
    }
}

impl TextureProvider for BitmapTextureProvider {
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let width = self.bitmap.info.width;
        let height = self.bitmap.info.height;
        let format = match self.bitmap.info.format {
            crate::core::image::ImageFormat::RGBA8888 => wgpu::TextureFormat::Rgba8Unorm,
            crate::core::image::ImageFormat::BGRA8888 => wgpu::TextureFormat::Bgra8Unorm,
            crate::core::image::ImageFormat::RGBX8888 => wgpu::TextureFormat::Rgba8Unorm,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[format],
        });

        queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: Default::default(),
            },
            &self.bitmap.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.bitmap.bytes_per_row),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.texture = Some(Rc::new(texture));
    }

    fn get_texture(&self) -> Option<Rc<wgpu::Texture>> {
        self.texture.clone()
    }

    fn get_width(&self) -> u32 {
        self.bitmap.info.width
    }

    fn get_height(&self) -> u32 {
        self.bitmap.info.height
    }

    fn get_format(&self) -> ImageFormat {
        self.bitmap.info.format
    }

    fn is_premutied(&self) -> bool {
        self.bitmap.info.premultiplied
    }
}

struct DirectTextureProvider {
    texture: Rc<wgpu::Texture>,
    info: ImageInfo,
}

impl DirectTextureProvider {
    fn new(texture: Rc<wgpu::Texture>, info: ImageInfo) -> Self {
        Self { texture, info }
    }
}

impl TextureProvider for DirectTextureProvider {
    fn prepare(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    fn get_texture(&self) -> Option<Rc<wgpu::Texture>> {
        Some(self.texture.clone())
    }

    fn get_width(&self) -> u32 {
        self.info.width
    }

    fn get_height(&self) -> u32 {
        self.info.height
    }

    fn get_format(&self) -> ImageFormat {
        self.info.format
    }

    fn is_premutied(&self) -> bool {
        self.info.premultiplied
    }
}

pub(crate) struct TextureFragment {
    transform: TransformGroup,
    texture: Box<dyn TextureProvider>,
    sampler: Option<wgpu::Sampler>,
    image_transform: Matrix4<f32>,

    image_transform_range: Range<wgpu::BufferAddress>,
    info_range: Range<wgpu::BufferAddress>,
}

impl TextureFragment {
    pub(crate) fn new_with_bitmap(
        vw: f32,
        vh: f32,
        transform: Matrix4<f32>,
        bitmap: Rc<Bitmap>,
        image_transform: Matrix4<f32>,
    ) -> Self {
        Self {
            transform: TransformGroup::new(
                Matrix4::new_orthographic(0.0, vw, vh, 0.0, -1000.0, 1000.0),
                transform,
                Vector4::new(0.0, 0.0, 0.0, 0.0),
            ),
            texture: Box::new(BitmapTextureProvider::new(bitmap)),
            sampler: None,
            image_transform,
            image_transform_range: 0..0,
            info_range: 0..0,
        }
    }

    pub(crate) fn new_with_texture(
        vw: f32,
        vh: f32,
        transform: Matrix4<f32>,
        texture: Rc<wgpu::Texture>,
        info: ImageInfo,
        image_transform: Matrix4<f32>,
    ) -> Self {
        Self {
            transform: TransformGroup::new(
                Matrix4::new_orthographic(0.0, vw, vh, 0.0, -1000.0, 1000.0),
                transform,
                Vector4::new(0.0, 0.0, 0.0, 0.0),
            ),
            texture: Box::new(DirectTextureProvider::new(texture, info)),
            sampler: None,
            image_transform,
            image_transform_range: 0..0,
            info_range: 0..0,
        }
    }
}

impl Fragment for TextureFragment {
    fn get_pipeline_name(&self) -> &'static str {
        TEXTURE_PIPELINE_NAME
    }

    fn prepare(
        &mut self,
        depth: f32,
        buffer: &mut StageBuffer,
        _context: &GPUContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.transform.prepare(depth, buffer);

        self.texture.prepare(device, queue);

        self.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1000.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        }));

        let mut image_transform_buffer = smallvec::SmallVec::<[f32; 20]>::new();
        let bounds = [
            self.texture.get_width() as f32,
            self.texture.get_height() as f32,
            0.0,
            0.0,
        ];
        image_transform_buffer.extend_from_slice(self.image_transform.as_slice());
        image_transform_buffer.extend_from_slice(&bounds);

        self.image_transform_range =
            buffer.push_data_align(bytemuck::cast_slice(image_transform_buffer.as_slice()));

        let info = [
            self.texture.is_premutied() as u32,
            self.texture.get_format() as u32,
            0,
            0,
        ];

        self.info_range = buffer.push_data_align(bytemuck::cast_slice(&info));
    }

    fn gen_bind_groups<'a>(
        &self,
        device: &wgpu::Device,
        buffer: &'a wgpu::Buffer,
        pipeline: &'a Pipeline,
    ) -> Vec<wgpu::BindGroup> {
        // group 1 color uniform
        let group1_layout = pipeline
            .get_group_layout(1)
            .expect("Texture pipeline not have group 1");

        let texture_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Group"),
            layout: &group1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset: self.image_transform_range.start,
                        size: wgpu::BufferSize::new(
                            self.image_transform_range.end - self.image_transform_range.start,
                        ),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset: self.info_range.start,
                        size: wgpu::BufferSize::new(self.info_range.end - self.info_range.start),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &self
                            .texture
                            .get_texture()
                            .expect("Texture not prepared")
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(
                        self.sampler.as_ref().expect("Texture not prepared"),
                    ),
                },
            ],
        });
        vec![
            self.gen_common_bind_groups(device, buffer, pipeline),
            texture_group,
        ]
    }

    fn gen_common_bind_groups<'a>(
        &self,
        device: &wgpu::Device,
        buffer: &'a wgpu::Buffer,
        pipeline: &'a Pipeline,
    ) -> wgpu::BindGroup {
        let group0_layout = pipeline
            .get_group_layout(0)
            .expect("common group at slot 0 can not be get!");

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("NonColor Common Group"),
            layout: &group0_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer,
                    offset: self.transform.get_buffer_range().start,
                    size: wgpu::BufferSize::new(
                        self.transform.get_buffer_range().end
                            - self.transform.get_buffer_range().start,
                    ),
                }),
            }],
        })
    }
}
