struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(3) color: vec4<f32>,
) -> VsOut {
    var out: VsOut;
    out.position = vec4<f32>(position.xy * 0.8, position.z * 0.2 + 0.5, 1.0);
    out.color = vec4<f32>(color.rgb * vec3<f32>(0.82, 0.58, 0.36), max(color.a, 1.0));
    return out;
}

@fragment
fn fs_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
