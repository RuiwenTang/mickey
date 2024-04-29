struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct UserMatrix {
    mvp: mat4x4<f32>,
    transform: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> transform: UserMatrix;

@group(1) @binding(0)
var<uniform> color: vec4<f32>;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = transform.mvp * transform.transform * vec4<f32>(vertex.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(color.rgb * color.a, color.a);
}
