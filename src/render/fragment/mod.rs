mod color_fragment;

pub(crate) use color_fragment::*;

/// Fragment responsible for generate the color data when rendering a shape.
/// It also controls the DepthStencilState and BlendState. When create pipeline.
pub(crate) trait Fragment {
    /// The depth stencil state. When create pipeline.
    fn depth_stencil_state(&self) -> wgpu::DepthStencilState;
    /// The blend state. When create pipeline.
    fn blend_state(&self) -> Option<wgpu::BlendState>;

    /// The bind group entry. for create pipeline layout
    /// All fragment shader use group 1.
    fn get_group_entry(&self) -> Vec<wgpu::BindGroupLayoutEntry>;

    /// Generate the group for the fragment.
    fn gen_group(
        &self,
        buffer: &mut crate::render::StageBuffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> crate::render::Group;

    fn stencil_state_name(&self) -> &'static str;
}

pub(crate) trait DepthStencilStateProvider {
    fn stencil_state() -> wgpu::DepthStencilState;

    fn name() -> &'static str;
}

pub(crate) struct NoneStencil;

impl DepthStencilStateProvider for NoneStencil {
    fn stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }
    }

    fn name() -> &'static str {
        "none_stencil"
    }
}
