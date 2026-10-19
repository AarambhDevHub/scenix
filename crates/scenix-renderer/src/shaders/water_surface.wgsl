// Fresnel water surface shader (scenix v1.8.0).
//
// The vertex stage displaces the patch with the same GPU Gems Gerstner
// formulation as `scenix-water::WaveField` (wave number `k = 2π/λ`, phase
// speed `c = √(g/k)`, displacement `(dx, dz) · a·cos(f)` plus `a·sin(f)`
// vertical with `a = steepness/k`), so CPU `height_at` and GPU sampling
// agree. `waves` rows pack `(dir.x, dir.z, steepness, wavelength)`.
//
// Reflection and refraction targets are optional screen-space samples: the
// vertex stage exports clip-position screen UVs (`src_uv`), the fragment
// stage warps them by the surface normal, and `params1.x/z` control both
// mix strengths. Foam follows `scenix-water::FoamParams`: renormalized
// smoothstep on the displaced crest height.

struct FrameUniform {
    view_projection: mat4x4<f32>,
    camera_position_frame: vec4<f32>,
    resolution: vec4<f32>,
};

struct ObjectUniform {
    world: mat4x4<f32>,
};

struct WaterUniform {
    deep_color: vec4<f32>,
    shallow_color: vec4<f32>,
    waves: array<vec4<f32>, 4>, // (dir.x, dir.z, steepness, wavelength)
    params0: vec4<f32>,         // (foam_threshold, foam_sharpness, base_reflectance, wave_count)
    params1: vec4<f32>,         // (reflection_strength, refraction_strength, 0, 0)
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
    @builtin(position) projected: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_position: vec3<f32>,
    @location(4) crest: f32,
    @location(5) src_uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> frame: FrameUniform;
@group(1) @binding(0) var<uniform> object: ObjectUniform;
@group(2) @binding(0) var<uniform> water: WaterUniform;
@group(2) @binding(1) var water_reflection: texture_2d<f32>;
@group(2) @binding(2) var water_refraction: texture_2d<f32>;
@group(2) @binding(3) var env_sampler: sampler;
@group(3) @binding(0) var<uniform> lights: LightUniform;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
) -> VsOut {
    var displaced = position;
    var wave_normal = vec3<f32>(0.0, 1.0, 0.0);
    var crest = 0.0;
    let time = frame.resolution.z;
    for (var i = 0; i < 4; i++) {
        let wave = water.waves[i];
        let steep = wave.z;
        let wavelength = wave.w;
        if (steep > 0.0001 && wavelength > 0.0001) {
            let dir = normalize(wave.xy);
            let k = 6.28318530717958 / wavelength;
            let c = sqrt(9.8 / k);
            let f = k * (dot(dir, vec2<f32>(position.x, position.z)) - c * time);
            let amplitude = steep / k;
            displaced.x += dir.x * amplitude * cos(f);
            displaced.z += dir.y * amplitude * cos(f);
            displaced.y += amplitude * sin(f);
            crest += steep * sin(f);
            wave_normal.x -= dir.x * steep * cos(f);
            wave_normal.z -= dir.y * steep * cos(f);
            wave_normal.y -= steep * sin(f);
        }
    }
    var out: VsOut;
    let world_position = object.world * vec4<f32>(displaced, 1.0);
    out.projected = frame.view_projection * world_position;
    out.color = color;
    out.normal = normalize((object.world * vec4<f32>(normalize(wave_normal), 0.0)).xyz);
    out.uv = uv;
    out.world_position = world_position.xyz;
    out.crest = crest;
    out.src_uv = out.projected.xy / max(out.projected.w, 0.001) * 0.5 + 0.5;
    return out;
}

fn foam_mask(crest: f32, threshold: f32, sharpness: f32) -> f32 {
    let span = max(1.0 - threshold, 0.0001);
    let normalized = clamp((crest - threshold) / span, 0.0, 1.0);
    let edge = clamp(normalized * max(sharpness, 0.1) * 0.5, 0.0, 1.0);
    return edge * edge * (3.0 - 2.0 * edge);
}

@fragment
fn fs_main(
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_position: vec3<f32>,
    @location(4) crest: f32,
    @location(5) src_uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    let view = normalize(frame.camera_position_frame.xyz - world_position);
    let n = normalize(normal);
    let depth_factor = clamp(world_position.y * 0.3 + 0.5, 0.0, 1.0);
    var tint = mix(water.deep_color.rgb, water.shallow_color.rgb, depth_factor);
    var alpha = mix(0.88, 0.68, depth_factor);

    let base_reflectance = clamp(water.params0.z, 0.0, 1.0);
    let cos_theta = clamp(dot(view, n), 0.001, 1.0);
    let fresnel = base_reflectance + (1.0 - base_reflectance) * pow(1.0 - cos_theta, 5.0);

    // Optional screen-space targets: warped by the surface normal.
    let distortion = n.xz * 0.05;
    if (water.params1.x > 0.001) {
        let reflected = textureSampleLevel(water_reflection, env_sampler, src_uv + distortion, 0.0);
        tint = mix(tint, reflected.rgb, clamp(water.params1.x, 0.0, 1.0) * fresnel * 2.4);
    } else {
        // Fallback sky reflection via ambient + directional color.
        let fallback = lights.ambient.rgb * 2.2 + lights.directional_color.rgb * 0.28;
        tint = mix(tint, fallback, clamp(fresnel * 1.8, 0.0, 0.85));
    }
    if (water.params1.y > 0.001) {
        let refracted = textureSampleLevel(water_refraction, env_sampler, src_uv - distortion, 0.0);
        tint = mix(tint, refracted.rgb, clamp(water.params1.y, 0.0, 1.0) * (1.0 - fresnel));
    }

    // Sun specular.
    let light_dir = normalize(-lights.directional_direction.xyz);
    let half_dir = normalize(view + light_dir);
    let specular = pow(max(dot(n, half_dir), 0.0), 120.0) * lights.directional_color.a * 1.4;
    tint += lights.directional_color.rgb * specular;

    // Foam along the wave crests (scenix-water FoamParams convention).
    if (water.params0.x < 0.999) {
        let mask = foam_mask(crest, water.params0.x, water.params0.y);
        tint = mix(tint, vec3<f32>(0.92), mask);
        alpha = min(alpha + mask * 0.3, 1.0);
    }

    return vec4<f32>(tint, clamp(alpha, 0.0, 1.0));
}
