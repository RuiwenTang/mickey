use crate::ShaderGenerator;

use super::{DepthStencilStateProvider, Fragment, Group};

pub(crate) struct StencilFragment<DS: DepthStencilStateProvider> {
    _ds: std::marker::PhantomData<DS>,
}

impl<DS: DepthStencilStateProvider> StencilFragment<DS> {
    pub(crate) fn new() -> Self {
        StencilFragment {
            _ds: std::marker::PhantomData,
        }
    }
}

impl<DS: DepthStencilStateProvider> Fragment for StencilFragment<DS> {
    fn depth_stencil_state(&self) -> wgpu::DepthStencilState {
        DS::stencil_state()
    }

    fn blend_state(&self) -> Option<wgpu::BlendState> {
        None
    }

    fn get_group_entry(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![]
    }

    fn gen_group(
        &self,
        _buffer: &mut crate::render::StageBuffer,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Option<Group> {
        None
    }

    fn stencil_state_name(&self) -> &'static str {
        DS::name()
    }
}

impl<DS: DepthStencilStateProvider> ShaderGenerator for StencilFragment<DS> {
    fn shader_name(&self) -> String {
        String::from("StencilFragment")
    }

    fn shader_code(&self) -> String {
        String::from(
            r#"

        @fragment
        fn fs_main() -> @location(0) vec4<f32> {
            return vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }

        "#,
        )
    }
}
