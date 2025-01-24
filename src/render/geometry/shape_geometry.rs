use std::num::NonZero;

use crate::{Geometry, Group, Matrix3x3, PositionChunk, ShaderGenerator, StageBuffer, Uniform};

pub(crate) struct ShapeGeometry {
    pos: PositionChunk,
}

impl ShapeGeometry {
    pub fn new(pos: PositionChunk) -> Self {
        ShapeGeometry { pos }
    }
}

impl Geometry for ShapeGeometry {
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
        vec![PositionChunk::group_entry()]
    }

    fn gen_group(&self, buffer: &mut StageBuffer) -> Group {
        let uniform = self.pos.upload(buffer);

        Group {
            index: 0,
            bindings: vec![uniform],
        }
    }
}

impl ShaderGenerator for ShapeGeometry {
    fn shader_name(&self) -> String {
        return String::from("SolidColorGeometry");
    }

    fn shader_code(&self) -> String {
        let mut code = PositionChunk::code_fragment();

        code += r#"
            @vertex
            fn vs_main(@location(0) pos: vec2<f32>) -> @builtin(position) vec4<f32> {
                return calculate_position(pos);
            }
        "#;

        return code;
    }
}

pub(crate) struct ShapeMatrixGeometry {
    pos: PositionChunk,
    matrix: Matrix3x3,
}

impl ShapeMatrixGeometry {
    pub fn new(pos: PositionChunk, matrix: Matrix3x3) -> Self {
        ShapeMatrixGeometry { pos, matrix }
    }
}

impl Geometry for ShapeMatrixGeometry {
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
                min_binding_size: NonZero::new(16 * 4 as wgpu::BufferAddress),
            },
            count: None,
        }]
    }

    fn gen_group(&self, buffer: &mut StageBuffer) -> Group {
        let uniform = self.pos.upload(buffer);

        let matrix_data: [f32; 16] = self.matrix.inverse().unwrap_or_default().into();

        let matrix = buffer.push_align(&matrix_data);
        Group {
            index: 0,
            bindings: vec![uniform, Uniform::Buffer(matrix, 1)],
        }
    }
}

impl ShaderGenerator for ShapeMatrixGeometry {
    fn shader_name(&self) -> String {
        String::from("ShapeMatrixGeometry")
    }

    fn shader_code(&self) -> String {
        let mut code = PositionChunk::code_fragment();

        code += r#"
            @group(0) @binding(1) var<uniform> local_matrix: mat4x4<f32>;

            struct VSOutput {
                @builtin(position) pos: vec4<f32>,
                @location(0) v_pos: vec2<f32>,
            }

            @vertex
            fn vs_main(@location(0) pos: vec2<f32>) -> VSOutput {
                var output: VSOutput;
                
                let v_pos = vec4<f32>(pos, 0.0, 1.0) * local_matrix;

                output.pos = calculate_position(pos);
                output.v_pos = v_pos.xy;

                return output;
            }
        "#;

        return code;
    }
}
