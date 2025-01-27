use std::num::NonZero;

use crate::{Matrix3x3, PositionChunk, ShaderGenerator, Uniform};

use super::{Geometry, Group, StageBuffer};

pub(crate) struct ImageGeometry {
    pos: PositionChunk,
    matrix: Matrix3x3,
    width: f32,
    height: f32,
}

impl ImageGeometry {
    pub fn new(pos: PositionChunk, matrix: Matrix3x3, width: f32, height: f32) -> Self {
        ImageGeometry {
            pos,
            matrix,
            width,
            height,
        }
    }
}

impl Geometry for ImageGeometry {
    fn get_vertex_attributes(&self) -> (Vec<wgpu::VertexAttribute>, wgpu::BufferAddress) {
        (
            vec![wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
            std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
        )
    }

    fn get_group_entry(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![PositionChunk::group_entry(), wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZero::new((4 + 16) * 4 as wgpu::BufferAddress),
            },
            count: None,
        }]
    }

    fn gen_group(&self, buffer: &mut StageBuffer) -> Group {
        let uniform = self.pos.upload(buffer);

        let mut data = vec![self.width, self.height, 0.0, 0.0];

        let matrix_data: [f32; 16] = self.matrix.inverse().unwrap_or_default().into();

        data.extend_from_slice(&matrix_data);

        let view = buffer.push_align(data.as_slice());

        Group {
            index: 0,
            bindings: vec![uniform, Uniform::Buffer(view, 1)],
        }
    }
}

impl ShaderGenerator for ImageGeometry {
    fn shader_name(&self) -> String {
        String::from("ImageGeometry")
    }

    fn shader_code(&self) -> String {
        let mut code = PositionChunk::code_fragment();

        code += r#"
            struct ImageBoundsInfo {
                bounds: vec4<f32>,
                matrix: mat4x4<f32>,
            }
        
            @group(0) @binding(1) var<uniform> image_info: ImageBoundsInfo;

            struct VSOutput {
                @builtin(position) pos  : vec4<f32>,
                @location(0) v_tex_coord: vec2<f32>,
            }

            @vertex
            fn vs_main(@location(0) pos: vec2<f32>) -> VSOutput {
                var output: VSOutput;
                output.pos = calculate_position(pos);

                var mapped_pos = (image_info.matrix * vec4<f32>(pos, 0.0, 1.0)).xy;

                var mapped_lt = vec2<f32>(0.0, 0.0);
                var mapped_rb = vec2<f32>(image_info.bounds.x, image_info.bounds.y);

                output.v_tex_coord = (mapped_pos - mapped_lt) / (mapped_rb - mapped_lt);

                return output;
            }
        "#;

        return code;
    }
}
