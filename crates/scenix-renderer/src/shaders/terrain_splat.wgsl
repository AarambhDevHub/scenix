// Splat-blended terrain shader (scenix v1.8.0 "Terrain, Sky, And Water").
//
// `layer_colors` and `layer_params` pack the four splat layers
// (grass/rock/dirt/sand): albedo RGBA plus `(roughness, tiling, has_detail,
// has_splat)` flags. Splat texture channels map `r = w0`, `g = w1`,
// `b = w2`, `a = w3`; the vertex-color fallback packs the same weights
// reversed `(r, g, b, a) = (w3, w2, w1, w0)` so textures and CPU geometry
// read from one convention each.

struct FrameUniform {
    view_projection: mat4x4<f32>,
    camera_position_frame: vec4<f32>,
    resolution: vec4<f32>,
};

struct ObjectUniform {
    world: mat4x4<f32>,
};

struct TerrainUniform {
    layer_colors: array<vec4<f32>, 4>,
    layer_params: array<vec4<f32>, 4>,
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
@group(2) @binding(0) var<uniform> terrain: TerrainUniform;
@group(2) @binding(1) var terrain_splat: texture_2d<f32>;
@group(2) @binding(2) var terrain_detail: texture_2d<f32>;
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
    out.normal = normalize((object.world * vec4<f32>(normal, 0.0)).xyz);
    out.uv = uv;
    out.world_position = world_position.xyz;
    return out;
}

fn base_color(weights: vec4<f32>) -> vec4<f32> {
    var color = weights.x * terrain.layer_colors[0];
    color += weights.y * terrain.layer_colors[1];
    color += weights.z * terrain.layer_colors[2];
    color += weights.w * terrain.layer_colors[3];
    return color;
}

@fragment
fn fs_main(
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_position: vec3<f32>,
) -> @location(0) vec4<f32> {
    let layer0 = terrain.layer_params[0];
    let has_splat = layer0.w > 0.5;
    var weights: vec4<f32>;
    if (has_splat) {
        // splat texture channels: r = w0, g = w1, b = w2, a = w3.
        weights = textureSampleLevel(terrain_splat, env_sampler, uv, 0.0);
    } else {
        // vertex colors pack (r, g, b, a) = (w3, w2, w1, w0).
        weights = vec4<f32>(color.a, color.b, color.g, color.r);
    }
    let total = max(weights.x + weights.y + weights.z + weights.w, 0.001);
    weights /= vec4<f32>(total);

    var albedo = base_color(weights).rgb;
    var roughness = weights.x * layer0.x
        + weights.y * terrain.layer_params[1].x
        + weights.z * terrain.layer_params[2].x
        + weights.w * terrain.layer_params[3].x;
    roughness = clamp(roughness, 0.04, 1.0);

    if (layer0.z > 0.5) {
        let tiling = vec2<f32>(layer0.y, layer0.y);
        let detail = textureSampleLevel(terrain_detail, env_sampler, uv * tiling, 0.0);
        albedo *= mix(vec3<f32>(1.0), detail.rgb, 0.65);
    }

    let ambient = lights.ambient.rgb + lights.environment.rgb * lights.environment.a;
    let light_dir = normalize(-lights.directional_direction.xyz);
    let diffuse = max(dot(normal, light_dir), 0.0) * lights.directional_color.a;
    if (has_splat && weights.x < 0.05) {
        // Below-horizon fading handled by scene lighting; no-op branch keeps
        // the sample-side weights stable.
    }
    let specular = (1.0 - roughness) * 0.16;
    var lit = albedo * (ambient + vec3<f32>(diffuse) + vec3<f32>(specular));
    if (lights.directional_direction.w > 0.5 && world_position.y < 0.04) {
        let contact = smoothstep(2.2, 0.1, length(world_position.xz));
        lit *= 1.0 - contact * 0.22;
    }
    return vec4<f32>(lit, 1.0);
}
