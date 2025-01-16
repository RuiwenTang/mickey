use std::fmt::format;

use crate::{Color, DepthStencilStateProvider, Fragment, Group, ShaderGenerator, Uniform};

pub(crate) struct ColorFragment<DS: DepthStencilStateProvider> {
    color: Color,
    _ds: std::marker::PhantomData<DS>,
}

impl<DS: DepthStencilStateProvider> ColorFragment<DS> {
    pub(crate) fn new(color: Color) -> Self {
        ColorFragment {
            color,
            _ds: std::marker::PhantomData,
        }
    }
}

impl<DS: DepthStencilStateProvider> Fragment for ColorFragment<DS> {
    fn depth_stencil_state(&self) -> wgpu::DepthStencilState {
        DS::stencil_state()
    }

    fn blend_state(&self) -> (wgpu::BlendState, wgpu::ColorWrites) {
        (
            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            wgpu::ColorWrites::ALL,
        )
    }

    fn get_group_entry(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }]
    }

    fn gen_group(
        &self,
        buffer: &mut crate::render::StageBuffer,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Group {
        let raw_color: [f32; 4] = self.color.clone().into();
        let view = buffer.push_align(&raw_color);

        Group {
            index: 1,
            bindings: vec![Uniform::Buffer(view, 0)],
        }
    }

    fn stencil_state_name(&self) -> &'static str {
        DS::name()
    }
}

impl<DS: DepthStencilStateProvider> ShaderGenerator for ColorFragment<DS> {
    fn shader_name(&self) -> String {
        return String::from("ColorFragment");
    }

    fn shader_code(&self) -> String {
        String::from(
            r#"
                @group(1) @binding(0) var<uniform> color: vec4<f32>;

                @fragment
                fn fs_main() -> @location(0) vec4<f32> {
                    return vec4<f32>(color.rgb * color.a, color.a);
                }
            "#,
        )
    }
}
