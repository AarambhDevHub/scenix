struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    values0: vec4<f32>,
    values1: vec4<f32>,
};

@group(0) @binding(0) var post_texture: texture_2d<f32>;
@group(0) @binding(1) var post_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );
    let position = positions[vertex_index];
    var out: VertexOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = position * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

fn luma(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn sample_color(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(post_texture, post_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)));
}

fn aces(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((color * (a * color + vec3<f32>(b))) / (color * (c * color + vec3<f32>(d)) + vec3<f32>(e)), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let color = sample_color(in.uv);
    let kind = u32(params.values1.x + 0.5);
    let texel = 1.0 / vec2<f32>(max(f32(textureDimensions(post_texture).x), 1.0), max(f32(textureDimensions(post_texture).y), 1.0));

    if kind == 1u {
        let threshold = params.values0.x;
        let intensity = params.values0.y;
        let radius = max(params.values0.z, 1.0);
        let bright = max(luma(color.rgb) - threshold, 0.0);
        let blur = (
            sample_color(in.uv + texel * vec2<f32>(radius, 0.0)).rgb +
            sample_color(in.uv - texel * vec2<f32>(radius, 0.0)).rgb +
            sample_color(in.uv + texel * vec2<f32>(0.0, radius)).rgb +
            sample_color(in.uv - texel * vec2<f32>(0.0, radius)).rgb
        ) * 0.25;
        return vec4<f32>(color.rgb + blur * bright * intensity, color.a);
    }

    if kind == 2u {
        let intensity = params.values0.y;
        let bias = params.values0.z;
        let shade = clamp(1.0 - intensity * 0.12 + bias, 0.0, 1.0);
        return vec4<f32>(color.rgb * shade, color.a);
    }

    if kind == 3u {
        let mode = params.values0.x;
        let exposure = params.values0.y;
        if mode < 0.5 {
            return color;
        }
        if mode < 1.5 {
            return vec4<f32>(color.rgb / (vec3<f32>(1.0) + color.rgb), color.a);
        }
        if mode < 2.5 {
            return vec4<f32>(aces(color.rgb), color.a);
        }
        return vec4<f32>(vec3<f32>(1.0) - exp(-color.rgb * exposure), color.a);
    }

    if kind == 4u || kind == 6u {
        let left = sample_color(in.uv - vec2<f32>(texel.x, 0.0)).rgb;
        let right = sample_color(in.uv + vec2<f32>(texel.x, 0.0)).rgb;
        let up = sample_color(in.uv + vec2<f32>(0.0, texel.y)).rgb;
        let down = sample_color(in.uv - vec2<f32>(0.0, texel.y)).rgb;
        let mixed = (left + right + up + down + color.rgb * 2.0) / 6.0;
        return vec4<f32>(mix(color.rgb, mixed, 0.35), color.a);
    }

    if kind == 5u {
        let feedback = params.values0.x;
        let jitter = params.values0.y;
        let shifted = sample_color(in.uv + texel * vec2<f32>(jitter, -jitter)).rgb;
        return vec4<f32>(mix(color.rgb, shifted, (1.0 - feedback) * 0.5), color.a);
    }

    if kind == 7u {
        let radius = max(params.values0.z, 0.0);
        let blur = (
            sample_color(in.uv + texel * vec2<f32>(radius, radius)).rgb +
            sample_color(in.uv + texel * vec2<f32>(-radius, radius)).rgb +
            sample_color(in.uv + texel * vec2<f32>(radius, -radius)).rgb +
            sample_color(in.uv + texel * vec2<f32>(-radius, -radius)).rgb
        ) * 0.25;
        return vec4<f32>(mix(color.rgb, blur, clamp(params.values0.y / 32.0, 0.0, 1.0)), color.a);
    }

    if kind == 8u {
        let fog_color = vec3<f32>(params.values0.x, params.values0.y, params.values0.z);
        let density = params.values0.w;
        return vec4<f32>(mix(color.rgb, fog_color, density), color.a);
    }

    if kind == 9u {
        let threshold = params.values0.z;
        let thickness = max(params.values0.w, 1.0);
        let edge = abs(luma(sample_color(in.uv + texel * vec2<f32>(thickness, 0.0)).rgb) - luma(color.rgb)) +
            abs(luma(sample_color(in.uv + texel * vec2<f32>(0.0, thickness)).rgb) - luma(color.rgb));
        let outline = vec4<f32>(params.values0.x, params.values0.y, params.values1.y, params.values1.z);
        return mix(color, outline, select(0.0, outline.a, edge > threshold));
    }

    if kind == 10u {
        let strength = params.values0.x;
        let shifted = sample_color(in.uv - texel * vec2<f32>(params.values0.y, 0.0) * strength).rgb;
        return vec4<f32>(mix(color.rgb, shifted, strength), color.a);
    }

    return color;
}
