mod color_fragment;
mod gradient_fragment;
mod stencil_fragment;

use std::marker::PhantomData;

pub(crate) use color_fragment::*;
pub(crate) use gradient_fragment::*;
pub(crate) use stencil_fragment::*;

use super::Group;

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
    ) -> Option<Group>;

    fn stencil_state_name(&self) -> &'static str;
}

pub(crate) trait DepthStencilStateProvider {
    fn stencil_state() -> wgpu::DepthStencilState;

    fn name() -> &'static str;
}

/// Indicate no need to do stencil test.
pub(crate) struct NoneStencil;

/// Indicate to mark stencil value by black and white.
pub(crate) struct BWStencilMask;

/// Indicate to do stencil test by non-zero.
pub(crate) struct NonZeroStencil;

/// Indicate to do stencil test by which not use stencil but depth.
pub(crate) struct StrokeStencil;

/// Indicate to do stencil test by even-odd.
pub(crate) struct EvenOddStencil;

/// Indicate to do stencil test and write depth value for intersect clip.
pub(crate) struct IntersectClip<T> {
    _stencil: PhantomData<T>,
}

/// Indicate to do stencil test and write depth value for difference clip.
pub(crate) struct DifferenceClip<T> {
    _stencil: PhantomData<T>,
}

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

impl DepthStencilStateProvider for BWStencilMask {
    fn stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::IncrementWrap,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::DecrementWrap,
                },
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: wgpu::DepthBiasState::default(),
        }
    }

    fn name() -> &'static str {
        "bw_stencil_mask"
    }
}

impl DepthStencilStateProvider for NonZeroStencil {
    fn stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: wgpu::DepthBiasState::default(),
        }
    }

    fn name() -> &'static str {
        "non_zero_stencil"
    }
}

impl DepthStencilStateProvider for EvenOddStencil {
    fn stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                read_mask: 0x01,
                write_mask: 0xff,
            },
            bias: wgpu::DepthBiasState::default(),
        }
    }

    fn name() -> &'static str {
        "even_odd_stencil"
    }
}

impl DepthStencilStateProvider for StrokeStencil {
    fn stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }
    }

    fn name() -> &'static str {
        "stroke_stencil"
    }
}

impl DepthStencilStateProvider for IntersectClip<NonZeroStencil> {
    fn stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Replace,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Replace,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: wgpu::DepthBiasState::default(),
        }
    }

    fn name() -> &'static str {
        "intersect_clip_winding"
    }
}

impl DepthStencilStateProvider for IntersectClip<EvenOddStencil> {
    fn stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Replace,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Replace,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                read_mask: 0x01,
                write_mask: 0xff,
            },
            bias: wgpu::DepthBiasState::default(),
        }
    }

    fn name() -> &'static str {
        "intersect_clip_even_odd"
    }
}

impl DepthStencilStateProvider for DifferenceClip<NonZeroStencil> {
    fn stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    fail_op: wgpu::StencilOperation::Replace,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    fail_op: wgpu::StencilOperation::Replace,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: wgpu::DepthBiasState::default(),
        }
    }

    fn name() -> &'static str {
        "difference_clip_winding"
    }
}

impl DepthStencilStateProvider for DifferenceClip<EvenOddStencil> {
    fn stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    fail_op: wgpu::StencilOperation::Replace,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    fail_op: wgpu::StencilOperation::Replace,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                read_mask: 0x01,
                write_mask: 0xff,
            },
            bias: wgpu::DepthBiasState::default(),
        }
    }

    fn name() -> &'static str {
        "difference_clip_even_odd"
    }
}
