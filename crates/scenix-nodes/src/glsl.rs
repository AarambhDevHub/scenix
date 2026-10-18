use alloc::format;
use alloc::string::{String, ToString};
use core::fmt;

use crate::graph::NodeDiagnostic;
use crate::material::NodeMaterial;
use crate::node::Stage;
use crate::post::PostNodeEffect;
use crate::value::NodeValueKind;
use crate::wgsl::{Engine, Needs, Scope};

/// Reserved configuration for GLSL generation; kept for backend parity with
/// [`crate::wgsl::WgslOptions`].
#[derive(Clone, Copy, Debug, Default)]
pub struct GlslOptions;

/// Generated GLSL ES 3.00 program pair (vertex + fragment sources).
///
/// Hosts bind attributes by name (`position`, `normal`, `uv`, `color`,
/// `tangent`; the post pair synthesizes vertices from `gl_VertexID`) and
/// uniforms by the flattened names declared in the sources. Node statements
/// are byte-identical to the WGSL output of [`crate::wgsl`]; only type
/// constructors and uniform accessors differ.
#[derive(Clone, Debug)]
pub struct NodeGlslProgram {
    label: Option<String>,
    vertex_source: String,
    fragment_source: String,
}

impl NodeGlslProgram {
    /// Returns the program label, when set.
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the vertex-stage GLSL source.
    #[inline]
    pub fn vertex_source(&self) -> &str {
        &self.vertex_source
    }

    /// Returns the fragment-stage GLSL source.
    #[inline]
    pub fn fragment_source(&self) -> &str {
        &self.fragment_source
    }
}

impl fmt::Display for NodeGlslProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("-- vertex --\n")?;
        f.write_str(&self.vertex_source)?;
        f.write_str("\n-- fragment --\n")?;
        f.write_str(&self.fragment_source)
    }
}

/// Generates the GLSL ES 3.00 program pair for a node material.
///
/// # Errors
/// Returns the first [`NodeDiagnostic`] produced by graph validation.
pub fn generate_material_glsl(
    material: &NodeMaterial,
    _options: &GlslOptions,
) -> Result<NodeGlslProgram, NodeDiagnostic> {
    let graph = material.graph();
    let resolved = graph.resolve(Stage::Fragment)?;
    let order = graph.fragment_order();

    let mut fs = Engine::new(false, Scope::Fragment, graph, &resolved);
    fs.compile_order(&order);
    let color_text = fs.color_port_text();
    let alpha_text = fs.alpha_port_text();
    let library = glsl_library(&fs.needs);
    let color_kind =
        crate::wgsl::port_resolved_kind(&resolved, graph.fragment_color(), NodeValueKind::Vec4);

    let mut vs = Engine::new(false, Scope::Vertex, graph, &resolved);
    let mut vertex_statements = String::new();
    let mut vertex_offset_text = String::new();
    if let Some(vertex_order) = graph.vertex_order() {
        vs.compile_order(&vertex_order);
        vertex_offset_text = vs.port_text(graph.vertex_offset());
        vertex_statements = vs.statements;
    }

    Ok(NodeGlslProgram {
        label: Some(material.name().to_string()),
        vertex_source: material_vertex_stage(&vertex_statements, &vertex_offset_text, &library),
        fragment_source: material_fragment_stage(
            &fs.statements,
            &color_text,
            &alpha_text,
            &library,
            color_kind,
        ),
    })
}

/// Generates the GLSL ES 3.00 program pair for a post node effect.
///
/// # Errors
/// Returns the first [`NodeDiagnostic`] produced by graph validation.
pub fn generate_post_glsl(
    effect: &PostNodeEffect,
    _options: &GlslOptions,
) -> Result<NodeGlslProgram, NodeDiagnostic> {
    let graph = effect.graph();
    let resolved = graph.resolve(Stage::Post)?;
    let order = graph.fragment_order();

    let mut fs = Engine::new(false, Scope::Post, graph, &resolved);
    fs.compile_order(&order);
    let color_text = fs.color_port_text();
    let color_kind =
        crate::wgsl::port_resolved_kind(&resolved, graph.fragment_color(), NodeValueKind::Vec4);

    Ok(NodeGlslProgram {
        label: Some(effect.name().to_string()),
        vertex_source: post_vertex_stage(),
        fragment_source: post_fragment_stage(
            &fs.statements,
            &color_text,
            &glsl_library(&fs.needs),
            color_kind,
        ),
    })
}

/// Flat uniform declarations shared by both material stages; hosts bind by
/// name and unused entries are valid GLSL.
fn push_glsl_uniforms(source: &mut String) {
    source.push_str("uniform mat4 view_projection;\n");
    source.push_str("uniform mat4 world;\n");
    source.push_str("uniform vec2 resolution;\n");
    source.push_str("uniform float time;\n");
    source.push_str("uniform vec3 camera_position;\n");
    source.push_str("uniform vec4 material_base_color;\n");
    source.push_str("uniform vec3 material_emissive;\n");
    source.push_str("uniform float material_metallic;\n");
    source.push_str("uniform float material_roughness;\n");
    source.push_str("uniform float material_alpha_cutoff;\n");
    source.push_str("uniform vec4 ambient;\n");
    source.push_str("uniform vec4 directional_direction;\n");
    source.push_str("uniform vec4 directional_color;\n");
    source.push_str("uniform vec4 point_position_range;\n");
    source.push_str("uniform vec4 point_color;\n");
    source.push_str("uniform vec4 environment;\n");
    source.push_str("uniform sampler2D material_albedo;\n");
}

/// Material GLSL vertex stage.
fn material_vertex_stage(statements: &str, offset_text: &str, library: &str) -> String {
    let mut source = String::with_capacity(1024);
    source.push_str("#version 300 es\n");
    source.push_str("precision highp float;\n\n");
    source.push_str("// scenix v1.6 node material — GLSL ES 3.00 (WebGL2), vertex stage\n\n");
    source.push_str("in vec3 position;\n");
    source.push_str("in vec3 normal;\n");
    source.push_str("in vec2 uv;\n");
    source.push_str("in vec4 color;\n");
    source.push_str("in vec4 tangent;\n\n");
    push_glsl_uniforms(&mut source);
    source.push_str("\nout vec4 v_color;\nout vec3 v_normal;\nout vec2 v_uv;\nout vec3 v_world_position;\nout vec4 v_tangent;\nout vec3 v_position;\n\n");
    source.push_str(library);
    source.push_str("\nvoid main() {\n");
    source.push_str("    vec3 displaced = position;\n");
    source.push_str(statements);
    if !offset_text.is_empty() {
        source.push_str(&format!("    displaced = displaced + {};\n", offset_text));
    }
    source.push_str("    vec4 world4 = world * vec4(displaced, 1.0);\n");
    source.push_str("    gl_Position = view_projection * world4;\n");
    source.push_str("    v_color = color;\n");
    source.push_str("    v_normal = normalize((world * vec4(normal, 0.0)).xyz);\n");
    source.push_str("    v_uv = uv;\n");
    source.push_str("    v_world_position = world4.xyz;\n");
    source.push_str("    v_tangent = tangent;\n");
    source.push_str("    v_position = displaced;\n}\n");
    source
}

/// Material GLSL fragment stage.
fn material_fragment_stage(
    statements: &str,
    color_text: &str,
    alpha_text: &str,
    library: &str,
    color_kind: NodeValueKind,
) -> String {
    let mut source = String::with_capacity(1024);
    source.push_str("#version 300 es\n");
    source.push_str("precision highp float;\n\n");
    source.push_str("// scenix v1.6 node material — GLSL ES 3.00 (WebGL2), fragment stage\n\n");
    source.push_str("in vec4 v_color;\n");
    source.push_str("in vec3 v_normal;\n");
    source.push_str("in vec2 v_uv;\n");
    source.push_str("in vec3 v_world_position;\n");
    source.push_str("in vec4 v_tangent;\n");
    source.push_str("in vec3 v_position;\n\n");
    source.push_str("out vec4 frag_color;\n\n");
    push_glsl_uniforms(&mut source);
    source.push('\n');
    source.push_str(library);
    source.push_str("\nvoid main() {\n");
    source.push_str(statements);
    match color_kind {
        // A vec3 result (lighting, fog) must not lose its alpha through a
        // GLSL vec4(vec3) constructor, which zero-fills w.
        NodeValueKind::Vec3 => {
            source.push_str(&format!(
                "    vec4 final_color = vec4({}, {});\n",
                color_text, alpha_text
            ));
        }
        _ => {
            source.push_str(&format!(
                "    vec4 final_color = vec4({});\n    final_color.a = {};\n",
                color_text, alpha_text
            ));
        }
    }
    source.push_str("    if (material_alpha_cutoff >= 0.0 && final_color.a < material_alpha_cutoff) {\n        discard;\n    }\n");
    source.push_str("    frag_color = final_color;\n}\n");
    source
}

/// Post GLSL full-screen triangle vertex stage (mirrors the scenix post pass).
fn post_vertex_stage() -> String {
    let mut source = String::with_capacity(512);
    source.push_str("#version 300 es\n");
    source.push_str("precision highp float;\n\n");
    source.push_str("// scenix v1.6 post node effect — GLSL ES 3.00 (WebGL2), vertex stage\n\n");
    source.push_str("out vec2 v_uv;\n\nvoid main() {\n");
    source.push_str(
        "    vec2 positions[3] = vec2[3](vec2(-1.0, -3.0), vec2(3.0, 1.0), vec2(-1.0, 1.0));\n",
    );
    source.push_str("    vec2 position = positions[gl_VertexID];\n");
    source.push_str("    gl_Position = vec4(position, 0.0, 1.0);\n");
    source.push_str("    v_uv = position * vec2(0.5, -0.5) + vec2(0.5, 0.5);\n}\n");
    source
}

/// Post GLSL fragment stage.
fn post_fragment_stage(
    statements: &str,
    color_text: &str,
    library: &str,
    color_kind: NodeValueKind,
) -> String {
    let mut source = String::with_capacity(768);
    source.push_str("#version 300 es\n");
    source.push_str("precision highp float;\n\n");
    source.push_str("// scenix v1.6 post node effect — GLSL ES 3.00 (WebGL2), fragment stage\n\n");
    source.push_str("in vec2 v_uv;\n\nout vec4 frag_color;\n\n");
    source.push_str("uniform sampler2D post_texture;\n");
    source.push_str("uniform vec4 values0;\n");
    source.push_str("uniform vec4 values1;\n");
    source.push_str("uniform vec2 resolution;\n");
    source.push_str("uniform float time;\n\n");
    source.push_str(library);
    source.push_str("\nvoid main() {\n");
    source.push_str(statements);
    match color_kind {
        NodeValueKind::Vec3 => {
            source.push_str(&format!("    frag_color = vec4({}, 1.0);\n", color_text));
        }
        _ => {
            source.push_str(&format!("    frag_color = vec4({});\n", color_text));
        }
    }
    source
}

/// Conditional GLSL helper library (mirrors [`crate::wgsl`]'s WGSL library).
fn glsl_library(needs: &Needs) -> String {
    let mut source = String::new();
    if needs.any_lighting() {
        source.push_str(
            "vec3 scenix_light_lambert(vec3 base_color, vec3 n, vec3 world_position) {\n",
        );
        source.push_str("    vec3 light_dir = normalize(-directional_direction.xyz);\n");
        source.push_str("    float diffuse = max(dot(n, light_dir), 0.0) * directional_color.a;\n");
        source.push_str("    vec3 to_point = point_position_range.xyz - world_position;\n");
        source.push_str("    float point_distance = max(length(to_point), 0.001);\n");
        source.push_str("    float point_range = point_position_range.w;\n");
        source.push_str("    float point_attenuation = point_range > 0.0 ? max(1.0 - point_distance / max(point_range, 0.001), 0.0) : 1.0 / (point_distance * point_distance);\n");
        source.push_str("    diffuse += max(dot(n, normalize(to_point)), 0.0) * point_color.a * point_attenuation;\n");
        source
            .push_str("    vec3 scene_ambient = ambient.rgb + environment.rgb * environment.a;\n");
        source.push_str("    return base_color * (scene_ambient + vec3(diffuse));\n}\n\n");
    }
    if needs.phong {
        source
            .push_str("vec3 scenix_light_phong(vec3 base_color, vec3 n, vec3 world_position) {\n");
        source.push_str("    vec3 lit = scenix_light_lambert(base_color, n, world_position);\n");
        source.push_str("    vec3 light_dir = normalize(-directional_direction.xyz);\n");
        source.push_str("    vec3 view_dir = normalize(camera_position - world_position);\n");
        source.push_str("    vec3 half_dir = normalize(light_dir + view_dir);\n");
        source.push_str("    float roughness = max(material_roughness, 0.04);\n");
        source.push_str("    float spec_exp = 8.0 + (1.0 - roughness) * 88.0;\n");
        source.push_str(
            "    float specular = pow(max(dot(n, half_dir), 0.0), spec_exp) * (1.0 - roughness);\n",
        );
        source.push_str("    return lit + vec3(specular) * directional_color.rgb;\n}\n\n");
    }
    if needs.toon {
        source.push_str(
            "vec3 scenix_light_toon(vec3 base_color, vec3 n, vec3 world_position, float bands) {\n",
        );
        source.push_str("    vec3 lit = scenix_light_lambert(base_color, n, world_position);\n");
        source.push_str("    vec3 light_scale = max(lit, vec3(0.001));\n");
        source.push_str("    vec3 quantized = floor(light_scale * bands) / (bands - 1.0);\n");
        source.push_str("    return clamp(quantized, vec3(0.0), vec3(1.0));\n}\n\n");
    }
    if needs.tonemap_reinhard {
        source.push_str("vec3 scenix_tonemap_reinhard(vec3 color) {\n    return color / (vec3(1.0) + color);\n}\n\n");
    }
    if needs.tonemap_aces {
        source.push_str("vec3 scenix_tonemap_aces(vec3 color) {\n");
        source.push_str("    const float a = 2.51;\n    const float b = 0.03;\n    const float c = 2.43;\n    const float d = 0.59;\n    const float e = 0.14;\n");
        source.push_str("    return clamp((color * (a * color + vec3(b))) / (color * (c * color + vec3(d)) + vec3(e)), vec3(0.0), vec3(1.0));\n}\n\n");
    }
    if needs.tonemap_exposure {
        source.push_str("vec3 scenix_tonemap_exposure(vec3 color, float exposure) {\n    return vec3(1.0) - exp(-color * vec3(exposure));\n}\n\n");
    }
    if needs.srgb_to_linear || needs.linear_to_srgb {
        source.push_str("float scenix_srgb_channel_to_linear(float channel) {\n");
        source.push_str("    float low = channel / 12.92;\n");
        source.push_str("    float high = pow((channel + 0.055) / 1.055, 2.4);\n");
        source.push_str("    return channel <= 0.04045 ? low : high;\n}\n");
        source.push_str("float scenix_srgb_channel_from_linear(float channel) {\n");
        source.push_str("    float low = max(channel, 0.0) * 12.92;\n");
        source.push_str("    float high = 1.055 * pow(max(channel, 0.0), 1.0 / 2.4) - 0.055;\n");
        source.push_str("    return channel <= 0.0031308 ? low : high;\n}\n");
    }
    if needs.srgb_to_linear {
        source.push_str("vec3 scenix_srgb_to_linear(vec3 color) {\n");
        source.push_str("    return vec3(scenix_srgb_channel_to_linear(color.r), scenix_srgb_channel_to_linear(color.g), scenix_srgb_channel_to_linear(color.b));\n}\n");
    }
    if needs.linear_to_srgb {
        source.push_str("vec3 scenix_linear_to_srgb(vec3 color) {\n");
        source.push_str("    return vec3(scenix_srgb_channel_from_linear(color.r), scenix_srgb_channel_from_linear(color.g), scenix_srgb_channel_from_linear(color.b));\n}\n");
    }
    source
}
