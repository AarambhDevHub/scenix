// Particle billboard draw path (scenix v1.7.0).
//
// Each instance is one 64-byte record; the vertex shader expands a
// camera-facing quad from a six-corner corner table, so particle draw calls
// need no index buffer. Instances arrive instanced per-vertex:
//
//   location 0: center (xyz) + size (w)      — world-space; size is the
//                                             billboard half-extent
//   location 1: color (rgba), premultiplied by the material tint on CPU
//   location 2: uv_rect (u0, v0, u1, v1)     — scenix UV: v0 is the TOP edge
//   location 3: rotation (x)                 — radians around the view axis

struct FrameUniform {
    view_projection: mat4x4<f32>,
    camera_position_frame: vec4<f32>,
    resolution: vec4<f32>,
}

// Filled per frame from the renderer: camera billboard basis and diagnostics.
struct ParticleParams {
    camera_right: vec4<f32>, // xyz basis, w unused
    camera_up: vec4<f32>,    // xyz basis, w unused
    misc: vec4<f32>,         // (elapsed, aspect, dt, frame_index)
}

@group(0) @binding(0)
var<uniform> frame: FrameUniform;
@group(0) @binding(1)
var<uniform> params: ParticleParams;

@group(1) @binding(0)
var sprite_texture: texture_2d<f32>;
@group(1) @binding(1)
var sprite_sampler: sampler;

// Two CCW triangles; dynamic indexing needs a private var table in WGSL.
var<private> CORNERS = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
);

struct VertexIn {
    @location(0) center_size: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv_rect: vec4<f32>,
    @location(3) rotation: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) corner_index: u32, vertex: VertexIn) -> VertexOut {
    let corner = CORNERS[corner_index];
    // Rotate the quad inside the billboard plane before world placement;
    // sin/cos are cheap enough at particle counts scenix targets.
    let s = sin(vertex.rotation.x);
    let c = cos(vertex.rotation.x);
    let rotated = vec2<f32>(
        corner.x * c - corner.y * s,
        corner.x * s + corner.y * c,
    );
    let half_size = max(vertex.center_size.w, 0.0);
    let world = vertex.center_size.xyz
        + params.camera_right.xyz * rotated.x * half_size
        + params.camera_up.xyz * rotated.y * half_size;

    // Horizontals interpolate u across the cell; verticals map geometry
    // bottom (corner.y == -1) to the cell's BOTTOM edge (uv_rect.w) because
    // scenix samples textures with row 0 at v == 0.
    let fx = (corner.x + 1.0) * 0.5;
    let fy = (corner.y + 1.0) * 0.5;
    let uv_y = mix(vertex.uv_rect.w, vertex.uv_rect.y, fy);

    var out: VertexOut;
    out.clip_position = frame.view_projection * vec4<f32>(world, 1.0);
    out.uv = vec2<f32>(mix(vertex.uv_rect.x, vertex.uv_rect.z, fx), uv_y);
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(vertex: VertexOut) -> @location(0) vec4<f32> {
    let sprite = textureSample(sprite_texture, sprite_sampler, vertex.uv);
    // Additive materials arrive with (0, One) blending; alpha materials use
    // (SrcAlpha, OneMinusSrcAlpha) — both modulate the sprite textel.
    return vertex.color * sprite;
}
