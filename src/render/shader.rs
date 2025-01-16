use std::num::NonZero;

use crate::{StageBuffer, Uniform};

pub(crate) struct PositionChunk {
    mvp: [f32; 16],
    transform: [f32; 16],
    info: [f32; 4],
}

impl PositionChunk {
    pub(crate) fn new(mvp: [f32; 16], transform: [f32; 16], info: [f32; 4]) -> Self {
        Self {
            mvp,
            transform,
            info,
        }
    }

    pub(crate) fn code_fragment() -> String {
        String::from(
            r#"
            struct UserTramsform {
                mvp: mat4x4<f32>,
                transform: mat4x4<f32>,
                info: vec4<f32>,
            }

            @group(0) @binding(0) var<uniform> user_transform: UserTramsform;

            fn calculate_position(pos: vec2<f32>) -> vec4<f32> {
                var p: vec4<f32> = user_transform.mvp * user_transform.transform * vec4<f32>(pos, 0.0, 1.0);

                return vec4<f32>(
                    p.x / p.w,
                    p.y / p.w,
                    user_transform.info.x,
                    1.0,
                );
            }
            
            "#,
        )
    }

    pub(crate) fn group_entry() -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZero::new(((16 + 16 + 4) * 4) as wgpu::BufferAddress),
            },
            count: None,
        }
    }

    pub(crate) fn upload(&self, buffer: &mut StageBuffer) -> Uniform {
        let mut data: Vec<f32> = [self.mvp, self.transform].concat();

        data.extend_from_slice(&self.info);

        let view = buffer.push_align(data.as_slice());

        return Uniform::Buffer(view, 0);
    }
}
