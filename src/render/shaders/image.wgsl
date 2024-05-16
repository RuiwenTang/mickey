struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) vUV: vec2<f32>,
};

struct UserMatrix {
    mvp: mat4x4<f32>,
    transform: mat4x4<f32>,
    info: vec4<f32>,
};

struct ImageTransform {
    matrix: mat4x4<f32>,
    // [sizeX, sizeY, dummy, dummy]    
    bounds: vec4<f32>,
}

struct ImageInfo {
    // [alpha_type, color_type, dummy, dummy]
    info: vec4<u32>,
};

@group(0) @binding(0)
var<uniform> transform: UserMatrix;

@group(1) @binding(0)
var<uniform> uvTransform: ImageTransform;

@group(1) @binding(1)
var<uniform> imageInfo: ImageInfo;

@group(1) @binding(2) 
var image: texture_2d<f32>;

@group(1) @binding(3)
var imageSampler: sampler;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos: vec4<f32> = transform.mvp * transform.transform * vec4<f32>(vertex.position, 0.0, 1.0);

    var vUV = (uvTransform.matrix * vec4<f32>(vertex.position, 0.0, 1.0)).xy;

    out.vUV = vec2<f32>(vUV.x / uvTransform.bounds.x, vUV.y / uvTransform.bounds.y);
    out.position = vec4<f32>(pos.x / pos.w, pos.y / pos.w, transform.info[0], 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(image, imageSampler, in.vUV);

    if imageInfo.info[1] == 1 { // BGRA
        color = vec4<f32>(color.b, color.g, color.r, color.a);
    }

    if imageInfo.info[0] == 1 {
        return color;
    } else {
        return vec4<f32>(color.rgb * color.a, color.a);
    }
}