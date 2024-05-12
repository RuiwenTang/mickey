const MAX_COUNT: u32 = 16;
const STOP_COUNT = MAX_COUNT / 4;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) vPos: vec2<f32>,
};

struct UserMatrix {
    mvp: mat4x4<f32>,
    transform: mat4x4<f32>,
    info: vec4<f32>,
};

struct LinearInfo {
    // [p1.x, p1.y, p2.x, p2.y]
    pts: vec4<f32>,
    // [color_count, stops_count, dummy, dummy]
    counts: vec4<u32>,
};

struct ColorInfo {
    colors: array<vec4<f32>, MAX_COUNT>,
    stops: array<vec4<f32>, STOP_COUNT>,
};


@group(0) @binding(0) 
var<uniform> transform: UserMatrix;

@group(1) @binding(0)
var<uniform> gradientTransform: mat4x4<f32>;

@group(1) @binding(1)
var<uniform> linearInfo: LinearInfo;

@group(1) @binding(2)
var<uniform> colorInfo: ColorInfo;


@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos: vec4<f32> = transform.mvp * transform.transform * vec4<f32>(vertex.position, 0.0, 1.0);
    var vPos: vec4<f32> = gradientTransform * vec4<f32>(vertex.position.x, vertex.position.y, 0.0, 1.0);

    out.position = vec4<f32>(pos.x / pos.w, pos.y / pos.w, transform.info[0], 1.0);
    out.vPos = vPos.xy / vPos.w;
    return out;
}

fn gradient_stop(index: u32) -> f32 {
    if index >= MAX_COUNT {
        index = MAX_COUNT - 1;
    }

    var i: u32 = index / 4;
    var j: u32 = index % 4;

    return colorInfo.stops[i][j];
}

fn gradient_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    return clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
}

fn gradient_color(t: f32) -> vec4<f32> {
    var ret = vec4<f32>(0.0);

    var i: u32 = 0;
    var color_count = linearInfo.counts[0];
    var count: u32 = linearInfo.counts[1];

    var max_t = if count > 0 { gradient_stop(count - 1) } else { 1.0 };

    if t <= 0 {
        ret = colorInfo.colors[0];
    } else if t >= max_t {
        ret = colorInfo.colors[color_count - 1];
    } else {
        for (i = 0; i < color_count - 1; i++) {
            var stopi = if count > 0 { gradient_stop(i) } else {
                f32(i) / f32(color_count - 1)
            };
            var stopi1 = if count > 0 { gradient_stop(i + 1) } else {
                f32(i + 1) / f32(color_count - 1)
            };

            if t >= stopi && t < stopi1 {
                ret = colorInfo.colors[i] * (1.0 - gradient_step(stopi, stopi1, t));
                ret += colorInfo.colors[i + 1] * gradient_step(stopi, stopi1, t);
                break;
            }
        }
    }

    return ret;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pos: vec2<f32> = in.vPos;
    var st: vec2<f32> = linearInfo.pts.xy;
    var ed: vec2<f32> = linearInfo.pts.zw;

    var ba = ed - st;

    var t = abs(dot(pos - st, ba) / dot(ba, ba));

    // TODO: support tile mode
    var color = gradient_color(t);

    return vec4<f32>(color.rgb * color.a, color.a);
}