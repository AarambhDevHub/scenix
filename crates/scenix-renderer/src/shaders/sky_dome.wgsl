// Procedural sky background shader (scenix v1.8.0).
//
// Gradient bookends mirror `scenix_sky::GroundedSkybox`: the horizon color
// lands at `y = 0`, the sky lerps up to `top_color`, and the grounded band
// fades into `ground_color` across `[fade_start, fade_start + fade_range]`.
// The atmosphere path mixes a horizon haze and a sun glare disk; both use
// only the dome's own world-space direction so it can shade a flat backdrop
// or a sphere.

struct FrameUniform {
    view_projection: mat4x4<f32>,
    camera_position_frame: vec4<f32>,
    resolution: vec4<f32>,
};

struct ObjectUniform {
    world: mat4x4<f32>,
};

struct SkyUniform {
    top_color: vec4<f32>,
    horizon_color: vec4<f32>,
    ground_color: vec4<f32>,
    sun_params: vec4<f32>, // sun direction xyz + glare strength.
    sun_color: vec4<f32>,
    fade_params: vec4<f32>, // fade_start, fade_range, atmosphere flag, pad.
    tail: array<vec4<f32>, 2>, // pads the struct to 128 bytes.
};

struct LightUniform {
    ambient: vec4<f32>,
    directional_direction: vec4<f32>,
    directional_color: vec4<f32>,
    point_position_range: vec4<f32>,
    point_color: vec4<f32>,
    spot_direction_angle: vec4<f32>,
    spot_color: vec4<f32>,
    environment: vec4<f32>,
    counts: vec4<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_position: vec3<f32>,
};

@group(0) @binding(0) var<uniform> frame: FrameUniform;
@group(1) @binding(0) var<uniform> object: ObjectUniform;
@group(2) @binding(0) var<uniform> sky: SkyUniform;
@group(2) @binding(1) var sky_texture: texture_2d<f32>;
@group(2) @binding(2) var sky_texture_b: texture_2d<f32>;
@group(2) @binding(3) var env_sampler: sampler;
@group(3) @binding(0) var<uniform> lights: LightUniform;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
) -> VsOut {
    var out: VsOut;
    let world_position = object.world * vec4<f32>(position, 1.0);
    out.position = frame.view_projection * world_position;
    out.color = color;
    out.normal = normal;
    out.uv = uv;
    out.world_position = world_position.xyz;
    return out;
}

@fragment
fn fs_main(
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_position: vec3<f32>,
) -> @location(0) vec4<f32> {
    let view = normalize(world_position - frame.camera_position_frame.xyz);
    let horizon = sky.horizon_color.rgb;
    let top = sky.top_color.rgb;
    let ground = sky.ground_color.rgb;
    let fade_start = sky.fade_params.x;
    let atmosphere = sky.fade_params.z;
    let sun_dir = normalize(sky.sun_params.xyz);
    let glare = clamp(sky.sun_params.w, 0.0, 1.0);

    // Above-horizon ramp; mirrored below by the grounded fade band.
    var sky_col = mix(horizon, top, clamp(view.y * 1.6, 0.0, 1.0));
    if (view.y < 0.0) {
        let band = clamp((view.y - fade_start) / max(-fade_start, 0.001), 0.0, 1.0);
        sky_col = mix(ground, horizon, band);
    }

    if (atmosphere > 0.5) {
        // Haze accumulates near the horizon; the sun direction tints it.
        let up = clamp(view.y, 0.0, 1.0);
        let haze = pow(1.0 - up, 4.0) * 0.35;
        sky_col += haze * max(horizon, vec3<f32>(0.2));
        let sun_side = max(dot(view, sun_dir), 0.0);
        let day = clamp(sun_dir.y * 3.0 + 0.4, 0.0, 1.0);
        sky_col *= mix(vec3<f32>(0.32, 0.36, 0.42), vec3<f32>(1.0), day);
        // Glare: wide halo + tight sun disk, both scaled by card strength.
        sky_col += sky.sun_color.rgb * glare * (pow(sun_side, 12.0) * 0.22);
        sky_col += sky.sun_color.rgb * glare * (pow(sun_side, 180.0) * 0.85);
        sky_col += sky.sun_color.rgb * glare * (pow(sun_side, 1200.0) * 2.6);
    }

    return vec4<f32>(sky_col, 1.0);
}
