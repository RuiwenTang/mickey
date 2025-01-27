use std::{num::NonZero, rc::Rc};

use crate::{Image, ImageSource, ShaderGenerator, StageBuffer, Uniform};

use super::{DepthStencilStateProvider, Fragment, Group};

pub(crate) struct ImageFragment<DS: DepthStencilStateProvider> {
    pub(crate) image: Rc<Image>,
    pub(crate) sampler: Rc<wgpu::Sampler>,
    pub(crate) alpha: f32,
    pub(crate) premultiply_alpha: bool,

    _ds: std::marker::PhantomData<DS>,
}

impl<DS: DepthStencilStateProvider> ImageFragment<DS> {
    pub(crate) fn new(
        image: Rc<Image>,
        sampler: Rc<wgpu::Sampler>,
        alpha: f32,
        premultiply_alpha: bool,
    ) -> Self {
        Self {
            image,
            sampler,
            alpha,
            premultiply_alpha,
            _ds: std::marker::PhantomData,
        }
    }

    fn get_texture(
        &self,
        image: &Image,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Rc<wgpu::Texture> {
        match &image.source {
            ImageSource::Texture(tex, _) => tex.clone(),
            ImageSource::Bitmap(bitmap) => {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Image Texture"),
                    size: wgpu::Extent3d {
                        width: bitmap.info.width,
                        height: bitmap.info.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: bitmap.info.format.into(),
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &bitmap.data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bitmap.bytes_per_row),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: bitmap.info.width,
                        height: bitmap.info.height,
                        depth_or_array_layers: 1,
                    },
                );

                Rc::new(texture)
            }
        }
    }
}

impl<DS: DepthStencilStateProvider> Fragment for ImageFragment<DS> {
    fn depth_stencil_state(&self) -> wgpu::DepthStencilState {
        DS::stencil_state()
    }

    fn blend_state(&self) -> Option<wgpu::BlendState> {
        Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING)
    }

    fn get_group_entry(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZero::new(16 as wgpu::BufferAddress),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]
    }

    fn gen_group(
        &self,
        buffer: &mut StageBuffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Group> {
        let data: [f32; 4] = [self.alpha, 0.0, 0.0, 0.0];
        let view = buffer.push_align(&data);

        let texture = self.get_texture(&self.image, device, queue);

        let text_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Some(Group {
            index: 1,
            bindings: vec![
                Uniform::Buffer(view, 0),
                Uniform::Texture(Rc::new(text_view), 1),
                Uniform::Sampler(self.sampler.clone(), 2),
            ],
        })
    }

    fn stencil_state_name(&self) -> &'static str {
        DS::name()
    }
}

impl<DS: DepthStencilStateProvider> ShaderGenerator for ImageFragment<DS> {
    fn shader_name(&self) -> String {
        format!("ImageFragment_{}", self.premultiply_alpha)
    }
    fn shader_code(&self) -> String {
        let mut code = String::new();

        if !self.premultiply_alpha {
            code += r#"
                fn process_color(color: vec4<f32>) -> vec4<f32> {
                    return vec4<f32>(color.rgb * color.a, color.a);
                }
            "#;
        } else {
            code += r#"
                fn process_color(color: vec4<f32>) -> vec4<f32> {
                    return color;
                }
            "#;
        }

        code += r#"
                @group(1) @binding(0) var<uniform>  ext_info    : vec4<f32>;
                @group(1) @binding(1) var           texture     : texture_2d<f32>;
                @group(1) @binding(2) var           img_sampler : sampler;
                
                @fragment
                fn fs_main(@location(0) v_tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
                    var color = textureSample(texture, img_sampler, v_tex_coord);

                    color = process_color(color);
                    
                    return color * ext_info.x;
                }
            "#;

        return code;
    }
}
