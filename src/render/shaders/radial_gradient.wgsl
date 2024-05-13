const MAX_COUNT: u32 = 16;
const STOP_COUNT = MAX_COUNT / 4;

const TILE_MODE_CLAMP: u32 = 0;
const TILE_MODE_REPEAT: u32 = 1;
const TILE_MODE_MIRROR: u32 = 2;
const TILE_MODE_DECAL: u32 = 3;

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

struct RadialInfo {
    // [p1.x, p1.y, radius, dummy]
    pts: vec4<f32>,
};

struct ColorInfo {
    // [color_count, stops_count, tile_mode, dummy]
    counts: vec4<u32>,
    colors: array<vec4<f32>, MAX_COUNT>,
    stops: array<vec4<f32>, STOP_COUNT>,
};


@group(0) @binding(0) 
var<uniform> transform: UserMatrix;

@group(1) @binding(0)
var<uniform> gradientTransform: mat4x4<f32>;

@group(1) @binding(1)
var<uniform> colorInfo: ColorInfo;

@group(1) @binding(2)
var<uniform> radialInfo: RadialInfo;


@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos: vec4<f32> = transform.mvp * transform.transform * vec4<f32>(vertex.position, 0.0, 1.0);
    var vPos: vec4<f32> = gradientTransform * vec4<f32>(vertex.position.x, vertex.position.y, 0.0, 1.0);

    out.position = vec4<f32>(pos.x / pos.w, pos.y / pos.w, transform.info[0], 1.0);
    out.vPos = vPos.xy / vPos.w;
    return out;
}

fn remap_t(t: f32, mode: u32) -> f32 {
    if mode == TILE_MODE_CLAMP {
        return clamp(t, 0.0, 1.0);
    } else if mode == TILE_MODE_REPEAT {
        return fract(t);
    } else if mode == TILE_MODE_MIRROR {
        var t1 = t - 1.0;
        var t2 = t1 - 2.0 * floor(t1 * 0.5) - 1.0;
        return abs(t2);
    }

    return t;
}

fn gradient_stop(index: u32) -> f32 {
    var relIndex = index;
    if relIndex >= MAX_COUNT {
        relIndex = MAX_COUNT - 1;
    }

    var i: u32 = relIndex / 4;
    var j: u32 = relIndex % 4;

    return colorInfo.stops[i][j];
}

fn gradient_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    return clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
}

fn gradient_color(t: f32) -> vec4<f32> {
    var ret = vec4<f32>(0.0);

    var color_count = colorInfo.counts[0];
    var count: u32 = colorInfo.counts[1];

    var max_t = 1.0;
    if count > 0 { max_t = gradient_stop(count - 1); }

    if t <= 0 {
        ret = colorInfo.colors[0];
    } else if t >= max_t {
        ret = colorInfo.colors[color_count - 1];
    } else {
        for (var i: u32 = 0; i < color_count - 1; i++) {
            var stopi = f32(i) / f32(color_count - 1);

            if count > 0 { stopi = gradient_stop(i); }

            var stopi1 = f32(i + 1) / f32(color_count - 1);

            if count > 0 { stopi1 = gradient_stop(i + 1); }

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

    var radius = radialInfo.pts[2];

    var dist = distance(pos, radialInfo.pts.xy);

    var t = dist / radius;

    t = remap_t(t, colorInfo.counts[2]);

    var color = gradient_color(t);

    return vec4<f32>(color.rgb * color.a, color.a);
}