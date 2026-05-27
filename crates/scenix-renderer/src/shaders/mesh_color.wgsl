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

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
};

@group(0) @binding(0) var<uniform> frame: FrameUniform;
@group(1) @binding(0) var<uniform> object: ObjectUniform;
@group(2) @binding(0) var<uniform> material: MaterialUniform;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(3) color: vec4<f32>,
) -> VsOut {
    var out: VsOut;
    let world_position = object.world * vec4<f32>(position, 1.0);
    let world_normal = normalize((object.world * vec4<f32>(normal, 0.0)).xyz);
    out.position = frame.view_projection * world_position;
    out.color = material.base_color * vec4<f32>(color.rgb, max(color.a, 1.0));
    out.normal = world_normal;
    return out;
}

@fragment
fn fs_main(
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
) -> @location(0) vec4<f32> {
    let shader_code = material.params.z;
    let n = normalize(normal);
    if (shader_code > 5.5) {
        return vec4<f32>(n * 0.5 + vec3<f32>(0.5), color.a);
    }

    let light_dir = normalize(vec3<f32>(-0.35, 0.8, 0.45));
    var diffuse = max(dot(n, light_dir), 0.0);
    if (shader_code > 3.5 && shader_code < 4.5) {
        diffuse = floor(diffuse * 4.0) / 3.0;
    }
    let ambient = 0.22;
    let lit = color.rgb * (ambient + diffuse * 0.78) + material.emissive_cutoff.rgb;
    let wire_boost = select(vec3<f32>(1.0), vec3<f32>(1.22), shader_code > 4.5 && shader_code < 5.5);
    return vec4<f32>(lit * wire_boost, color.a);
}
