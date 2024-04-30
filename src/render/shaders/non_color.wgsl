struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct UserMatrix {
    mvp: mat4x4<f32>,
    transform: mat4x4<f32>,
    info: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> transform: UserMatrix;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos: vec4<f32> = transform.mvp * transform.transform * vec4<f32>(vertex.position, 0.0, 1.0);

    out.position = vec4<f32>(pos.x / pos.w, pos.y / pos.w, transform.info[0], 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> void {}