struct PickFrameUniform {
    view_projection: mat4x4<f32>,
};

struct PickObjectUniform {
    world: mat4x4<f32>,
    id_bytes: vec4<u32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

struct FragmentOutput {
    @location(0) object_id: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) depth: f32,
};

@group(0) @binding(0) var<uniform> frame: PickFrameUniform;
@group(1) @binding(0) var<uniform> object: PickObjectUniform;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    let world_position = object.world * vec4<f32>(position, 1.0);
    output.position = frame.view_projection * world_position;
    output.normal = normalize((object.world * vec4<f32>(normal, 0.0)).xyz);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    output.object_id = vec4<f32>(object.id_bytes) / 255.0;
    output.normal = vec4<f32>(normalize(input.normal) * 0.5 + vec3<f32>(0.5), 1.0);
    output.depth = input.position.z;
    return output;
}
