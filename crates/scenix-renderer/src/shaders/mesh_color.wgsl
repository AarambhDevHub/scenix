struct FrameUniform {
    view_projection: mat4x4<f32>,
    camera_position_frame: vec4<f32>,
    resolution: vec4<f32>,
};

struct ObjectUniform {
    world: mat4x4<f32>,
};

struct MaterialUniform {
    base_color: vec4<f32>,
    emissive_cutoff: vec4<f32>,
    params: vec4<f32>,
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
@group(2) @binding(0) var<uniform> material: MaterialUniform;
@group(2) @binding(1) var material_sampler: sampler;
@group(2) @binding(2) var material_albedo: texture_2d<f32>;
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
    let world_normal = normalize((object.world * vec4<f32>(normal, 0.0)).xyz);
    out.position = frame.view_projection * world_position;
    out.color = material.base_color * vec4<f32>(color.rgb, max(color.a, 1.0));
    out.normal = world_normal;
    out.uv = uv;
    out.world_position = world_position.xyz;
    return out;
}

fn has_feature(bits: f32, flag: f32) -> bool {
    let divided = floor(bits / flag);
    return divided - floor(divided * 0.5) * 2.0 >= 1.0;
}

@fragment
fn fs_main(
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_position: vec3<f32>,
) -> @location(0) vec4<f32> {
    let shader_code = material.params.z;
    let feature_bits = material.params.w;
    let n = normalize(normal);
    if (shader_code > 5.5) {
        return vec4<f32>(n * 0.5 + vec3<f32>(0.5), color.a);
    }

    var base = color;
    if (has_feature(feature_bits, 2.0)) {
        base *= textureSample(material_albedo, material_sampler, uv);
    }
    if (material.emissive_cutoff.a >= 0.0 && base.a < material.emissive_cutoff.a) {
        discard;
    }

    let light_dir = normalize(-lights.directional_direction.xyz);
    var diffuse = max(dot(n, light_dir), 0.0) * lights.directional_color.a;
    let to_point = lights.point_position_range.xyz - world_position;
    let point_distance = max(length(to_point), 0.001);
    let point_range = lights.point_position_range.w;
    let point_attenuation = select(
        1.0 / (point_distance * point_distance),
        max(1.0 - point_distance / max(point_range, 0.001), 0.0),
        point_range > 0.0,
    );
    diffuse += max(dot(n, normalize(to_point)), 0.0) * lights.point_color.a * point_attenuation;

    if (shader_code > 3.5 && shader_code < 4.5) {
        diffuse = floor(diffuse * 4.0) / 3.0;
    }
    let ambient = lights.ambient.rgb + lights.environment.rgb * lights.environment.a;
    let metallic = material.params.x;
    let roughness = max(material.params.y, 0.04);
    let specular_boost = (1.0 - roughness) * (0.16 + metallic * 0.5);
    let physical_boost = select(0.0, 0.08, shader_code > 0.5 && shader_code < 1.5);
    let unlit = shader_code > 1.5 && shader_code < 2.5;
    let lambert = shader_code > 2.5 && shader_code < 3.5;
    let light_scale = select(
        ambient + vec3<f32>(diffuse + specular_boost + physical_boost),
        vec3<f32>(1.0),
        unlit,
    );
    let lambert_scale = select(vec3<f32>(1.0), vec3<f32>(0.86), lambert);
    var lit = base.rgb * light_scale * lambert_scale + material.emissive_cutoff.rgb;
    if (lights.directional_direction.w > 0.5 && world_position.y < 0.04) {
        let contact = smoothstep(2.2, 0.1, length(world_position.xz));
        lit *= 1.0 - contact * 0.22;
    }
    let wire_boost = select(vec3<f32>(1.0), vec3<f32>(1.22), shader_code > 4.5 && shader_code < 5.5);
    return vec4<f32>(lit * wire_boost, base.a);
}
