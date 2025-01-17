use crate::{Geometry, Group, PositionChunk, ShaderGenerator, StageBuffer};

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
