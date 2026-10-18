use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use crate::graph::{NodeDiagnostic, ShaderGraph, ShaderNodeId, encode_kind_payload};
use crate::material::NodeMaterial;
use crate::node::{
    ComponentIndex, FogStyle, LightingStyle, NodeInput, ShaderNodeData, ShaderNodeKind, Stage,
    ToneMapKind, UniformInput, VertexAttribute,
};
use crate::post::PostNodeEffect;
use crate::value::{NodeValue, NodeValueKind};

/// Configuration for WGSL generation.
#[derive(Clone, Copy, Debug)]
pub struct WgslOptions {
    /// Name of the generated vertex entry point.
    pub vertex_entry: &'static str,
    /// Name of the generated fragment entry point.
    pub fragment_entry: &'static str,
}

impl Default for WgslOptions {
    #[inline]
    fn default() -> Self {
        Self {
            vertex_entry: "vs_main",
            fragment_entry: "fs_main",
        }
    }
}

/// A generated WGSL program: material or post-processing stage.
#[derive(Clone, Debug)]
pub struct NodeWgslProgram {
    label: Option<String>,
    source: String,
}

impl NodeWgslProgram {
    /// Returns the program label, when set.
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the WGSL source.
    #[inline]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Consumes the program and returns the raw source.
    #[inline]
    pub fn into_source(self) -> String {
        self.source
    }
}

impl fmt::Display for NodeWgslProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.source)
    }
}

/// Compilation scopes covered by the shared statement engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scope {
    /// Material vertex stage.
    Vertex,
    /// Material fragment stage.
    Fragment,
    /// Full-screen post stage.
    Post,
}

/// Library helper demands discovered while compiling statements.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Needs {
    pub(crate) lambert: bool,
    pub(crate) phong: bool,
    pub(crate) toon: bool,
    pub(crate) tonemap_none: bool,
    pub(crate) tonemap_reinhard: bool,
    pub(crate) tonemap_aces: bool,
    pub(crate) tonemap_exposure: bool,
    pub(crate) srgb_to_linear: bool,
    pub(crate) linear_to_srgb: bool,
}

impl Needs {
    /// Whether any lighting helper is referenced.
    pub(crate) fn any_lighting(&self) -> bool {
        self.lambert || self.phong || self.toon
    }
}

/// Shared statement compiler used by the WGSL and GLSL backends.
///
/// Statements are emitted in deterministic evaluation order with
/// common-subexpression elimination: nodes with identical (kind, inputs)
/// payloads reuse the earlier statement instead of re-emitting it.
pub(crate) struct Engine<'g, 'r> {
    wgsl: bool,
    scope: Scope,
    graph: &'g ShaderGraph,
    resolved: &'r BTreeMap<ShaderNodeId, Option<NodeValueKind>>,
    aliases: BTreeMap<ShaderNodeId, ShaderNodeId>,
    seen: BTreeMap<Vec<u8>, ShaderNodeId>,
    /// Lowered statements in evaluation order.
    pub statements: String,
    /// Library demands discovered during compilation.
    pub needs: Needs,
}

impl<'g, 'r> Engine<'g, 'r> {
    /// Creates a statement compiler for one shader scope.
    pub(crate) fn new(
        wgsl: bool,
        scope: Scope,
        graph: &'g ShaderGraph,
        resolved: &'r BTreeMap<ShaderNodeId, Option<NodeValueKind>>,
    ) -> Self {
        Self {
            wgsl,
            scope,
            graph,
            resolved,
            aliases: BTreeMap::new(),
            seen: BTreeMap::new(),
            statements: String::new(),
            needs: Needs::default(),
        }
    }

    /// Compiles every node in the given evaluation order.
    pub(crate) fn compile_order(&mut self, order: &[ShaderNodeId]) {
        for id in order {
            let Some(node) = self.graph.get(*id) else {
                continue;
            };
            self.compile(*id, node);
        }
    }

    fn compile(&mut self, id: ShaderNodeId, node: &ShaderNodeData) {
        let key = cse_key(node);
        if let Some(existing) = self.seen.get(&key) {
            self.aliases.insert(id, *existing);
            return;
        }
        self.seen.insert(key, id);
        self.track_needs(&node.kind);
        let expression = self.expression(node);
        let statement = if self.wgsl {
            format!("    let n{} = {};\n", id.0, expression)
        } else {
            let kind = self.kind_of(id);
            format!("    {} n{} = {};\n", kind.glsl_type(), id.0, expression)
        };
        self.statements.push_str(&statement);
    }

    fn track_needs(&mut self, kind: &ShaderNodeKind) {
        if matches!(self.scope, Scope::Vertex) {
            // Vertex statements never use library helpers.
            return;
        }
        match kind {
            ShaderNodeKind::Lighting(LightingStyle::Lambert) => self.needs.lambert = true,
            ShaderNodeKind::Lighting(LightingStyle::Phong) => self.needs.phong = true,
            ShaderNodeKind::Lighting(LightingStyle::Toon(_)) => self.needs.toon = true,
            ShaderNodeKind::ToneMap(ToneMapKind::None) => self.needs.tonemap_none = true,
            ShaderNodeKind::ToneMap(ToneMapKind::Reinhard) => self.needs.tonemap_reinhard = true,
            ShaderNodeKind::ToneMap(ToneMapKind::Aces) => self.needs.tonemap_aces = true,
            ShaderNodeKind::ToneMap(ToneMapKind::Exposure(_)) => self.needs.tonemap_exposure = true,
            ShaderNodeKind::SrgbToLinear => self.needs.srgb_to_linear = true,
            ShaderNodeKind::LinearToSrgb => self.needs.linear_to_srgb = true,
            _ => {}
        }
    }

    fn kind_of(&self, id: ShaderNodeId) -> NodeValueKind {
        let mut current = id;
        while let Some(next) = self.aliases.get(&current) {
            current = *next;
        }
        self.resolved
            .get(&current)
            .copied()
            .flatten()
            .unwrap_or(NodeValueKind::Float)
    }

    fn name_of(&self, id: ShaderNodeId) -> String {
        let mut current = id;
        while let Some(next) = self.aliases.get(&current) {
            current = *next;
        }
        format!("n{}", current.0)
    }

    // -- input resolution ------------------------------------------------------

    fn input(&self, node: &ShaderNodeData, slot: usize) -> (String, NodeValueKind) {
        match node.input(slot) {
            Some(NodeInput::Value(value)) => (self.literal(value), value.kind()),
            Some(NodeInput::Node(child)) => (self.name_of(*child), self.kind_of(*child)),
            None => (self.float_text(0.0), NodeValueKind::Float),
        }
    }

    fn literal(&self, value: &NodeValue) -> String {
        let prefix = |kind: NodeValueKind| ctor(kind, self.wgsl);
        match value {
            NodeValue::Float(value) => self.float_text(*value),
            NodeValue::Vec2(value) => format!(
                "{}{}, {})",
                prefix(NodeValueKind::Vec2),
                self.float_text(value.x),
                self.float_text(value.y)
            ),
            NodeValue::Vec3(value) => format!(
                "{}{}, {}, {})",
                prefix(NodeValueKind::Vec3),
                self.float_text(value.x),
                self.float_text(value.y),
                self.float_text(value.z)
            ),
            NodeValue::Vec4(value) => format!(
                "{}{}, {}, {}, {})",
                prefix(NodeValueKind::Vec4),
                self.float_text(value.x),
                self.float_text(value.y),
                self.float_text(value.z),
                self.float_text(value.w)
            ),
        }
    }

    fn float_text(&self, value: f32) -> String {
        if !value.is_finite() {
            // Non-finite baked constants clamp to zero at generation time.
            return if self.wgsl {
                "0f".to_string()
            } else {
                "0.0".to_string()
            };
        }
        let text = alloc::format!("{}", value);
        if self.wgsl {
            if text.contains('.') {
                format!("{}f", text)
            } else {
                format!("{}.0f", text)
            }
        } else if text.contains('.') {
            text
        } else {
            format!("{}.0", text)
        }
    }

    // -- expression builders -----------------------------------------------------

    fn expression(&self, node: &ShaderNodeData) -> String {
        match &node.kind {
            ShaderNodeKind::Constant(value) => self.literal(value),
            ShaderNodeKind::Uniform(uniform) => self.uniform_text(*uniform).to_string(),
            ShaderNodeKind::Attribute(attribute) => self.attribute_text(*attribute).to_string(),
            ShaderNodeKind::WorldPosition => "v_world_position".to_string(),
            ShaderNodeKind::WorldNormal => "v_normal".to_string(),
            ShaderNodeKind::ViewDirection => self.view_expression(),
            ShaderNodeKind::ScreenUv => self.screen_uv_expression(),
            ShaderNodeKind::TextureSample => {
                let (uv, _) = self.input(node, 0);
                self.sample_material(&uv)
            }
            ShaderNodeKind::Splat(target) => {
                let (value, _) = self.input(node, 0);
                format!("{}({})", ctor(*target, self.wgsl), value)
            }
            ShaderNodeKind::Component(index) => {
                let (value, _) = self.input(node, 0);
                format!("{}.{}", value, index.letter())
            }
            ShaderNodeKind::Add
            | ShaderNodeKind::Subtract
            | ShaderNodeKind::Multiply
            | ShaderNodeKind::Divide
            | ShaderNodeKind::Min
            | ShaderNodeKind::Max => {
                let (lhs, lhs_kind) = self.input(node, 0);
                let (rhs, rhs_kind) = self.input(node, 1);
                let result = if lhs_kind == rhs_kind || !lhs_kind.is_float() {
                    lhs_kind
                } else {
                    rhs_kind
                };
                let lhs = self.broadcast(&lhs, lhs_kind, result);
                let rhs = self.broadcast(&rhs, rhs_kind, result);
                match &node.kind {
                    ShaderNodeKind::Min => format!("min({}, {})", lhs, rhs),
                    ShaderNodeKind::Max => format!("max({}, {})", lhs, rhs),
                    _ => {
                        let symbol = match &node.kind {
                            ShaderNodeKind::Add => "+",
                            ShaderNodeKind::Subtract => "-",
                            ShaderNodeKind::Multiply => "*",
                            _ => "/",
                        };
                        format!("({} {} {})", lhs, symbol, rhs)
                    }
                }
            }
            ShaderNodeKind::Dot
            | ShaderNodeKind::Cross
            | ShaderNodeKind::Length
            | ShaderNodeKind::Normalize
            | ShaderNodeKind::Mix
            | ShaderNodeKind::Clamp
            | ShaderNodeKind::Saturate
            | ShaderNodeKind::Abs
            | ShaderNodeKind::Power
            | ShaderNodeKind::Sin
            | ShaderNodeKind::Cos
            | ShaderNodeKind::Fract
            | ShaderNodeKind::Floor
            | ShaderNodeKind::Step
            | ShaderNodeKind::Smoothstep => {
                let (a, a_kind) = self.input(node, 0);
                let (b, _) = self.input(node, 1);
                let (c, _) = if node.kind.port_count() > 2 {
                    self.input(node, 2)
                } else {
                    (String::new(), NodeValueKind::Float)
                };
                match &node.kind {
                    ShaderNodeKind::Dot => format!("dot({}, {})", a, b),
                    ShaderNodeKind::Cross => format!("cross({}, {})", a, b),
                    ShaderNodeKind::Length => format!("length({})", a),
                    ShaderNodeKind::Normalize => format!("normalize({}) ", a),
                    ShaderNodeKind::Mix => format!("mix({}, {}, {})", a, b, c),
                    ShaderNodeKind::Clamp => format!("clamp({}, {}, {})", a, b, c),
                    ShaderNodeKind::Saturate if a_kind.is_float() => {
                        format!("clamp({}, 0.0, 1.0)", a)
                    }
                    ShaderNodeKind::Saturate => format!(
                        "clamp({}, {}0.0), {}1.0))",
                        a,
                        ctor(a_kind, self.wgsl),
                        ctor(a_kind, self.wgsl)
                    ),
                    ShaderNodeKind::Abs => format!("abs({})", a),
                    ShaderNodeKind::Power => format!("pow({}, {})", a, b),
                    ShaderNodeKind::Sin => format!("sin({})", a),
                    ShaderNodeKind::Cos => format!("cos({})", a),
                    ShaderNodeKind::Fract => format!("fract({})", a),
                    ShaderNodeKind::Floor => format!("floor({})", a),
                    ShaderNodeKind::Step => format!("step({}, {})", a, b),
                    _ => format!("smoothstep({}, {}, {})", a, b, c),
                }
            }
            ShaderNodeKind::OneMinus => {
                let (value, _) = self.input(node, 0);
                format!("(1.0 - {})", value)
            }
            ShaderNodeKind::Remap => {
                let (x, _) = self.input(node, 0);
                let (from_min, _) = self.input(node, 1);
                let (from_max, _) = self.input(node, 2);
                let (to_min, _) = self.input(node, 3);
                let (to_max, _) = self.input(node, 4);
                format!(
                    "(({} - {}) / ({} - {})) * ({} - {}) + {}",
                    x, from_min, from_max, from_min, to_max, to_min, to_min
                )
            }
            ShaderNodeKind::Fresnel(power) => {
                let (normal, _) = self.input(node, 0);
                format!(
                    "pow(clamp(1.0 - abs(dot(normalize({}), {})), 0.0, 1.0), {})",
                    normal,
                    self.view_expression(),
                    self.float_text(*power)
                )
            }
            ShaderNodeKind::SrgbToLinear | ShaderNodeKind::LinearToSrgb => {
                let (color, color_kind) = self.input(node, 0);
                let rgb = match color_kind {
                    NodeValueKind::Vec4 => format!("{}.rgb", color),
                    _ => color,
                };
                if matches!(node.kind, ShaderNodeKind::SrgbToLinear) {
                    format!("scenix_srgb_to_linear({})", rgb)
                } else {
                    format!("scenix_linear_to_srgb({})", rgb)
                }
            }
            ShaderNodeKind::Lighting(style) => {
                let (color, color_kind) = self.input(node, 0);
                let base = match color_kind {
                    NodeValueKind::Vec4 => format!("{}.rgb", color),
                    _ => color,
                };
                match style {
                    LightingStyle::Lambert => format!(
                        "scenix_light_lambert({}, normalize(v_normal), v_world_position)",
                        base
                    ),
                    LightingStyle::Phong => format!(
                        "scenix_light_phong({}, normalize(v_normal), v_world_position)",
                        base
                    ),
                    LightingStyle::Toon(bands) => {
                        let bands = *bands;
                        format!(
                            "scenix_light_toon({}, normalize(v_normal), v_world_position, {})",
                            base,
                            self.float_text(bands.max(2) as f32)
                        )
                    }
                }
            }
            ShaderNodeKind::ToneMap(kind) => {
                let (color, color_kind) = self.input(node, 0);
                let rgb = match color_kind {
                    NodeValueKind::Vec4 => format!("{}.rgb", color),
                    _ => color,
                };
                match kind {
                    ToneMapKind::None => rgb,
                    ToneMapKind::Reinhard => format!("scenix_tonemap_reinhard({})", rgb),
                    ToneMapKind::Aces => format!("scenix_tonemap_aces({})", rgb),
                    ToneMapKind::Exposure(exposure) => format!(
                        "scenix_tonemap_exposure({}, {})",
                        rgb,
                        self.float_text(*exposure)
                    ),
                }
            }
            ShaderNodeKind::Fog(style) => self.fog_expression(style, node),
            ShaderNodeKind::PostSource => {
                let (uv, _) = self.input(node, 0);
                if self.wgsl {
                    format!("textureSample(post_texture, post_sampler, {})", uv)
                } else {
                    format!("texture(post_texture, clamp({}, vec2(0.0), vec2(1.0)))", uv)
                }
            }
            ShaderNodeKind::PostUv => "v_uv".to_string(),
            ShaderNodeKind::PostParam(index) => self.post_param_text(*index),
        }
    }

    fn fog_expression(&self, style: &FogStyle, node: &ShaderNodeData) -> String {
        let (color, color_kind) = self.input(node, 0);
        let rgb = match color_kind {
            NodeValueKind::Vec4 => format!("{}.rgb", color),
            _ => color,
        };
        let fog_color = match style {
            FogStyle::Exponential { color, .. } | FogStyle::Linear { color, .. } => {
                self.literal(color)
            }
        };
        let distance = format!(
            "distance(v_world_position, {})",
            self.uniform_text(UniformInput::CameraPosition)
        );
        let factor = match style {
            FogStyle::Exponential { density, .. } => format!(
                "1.0 - clamp(exp(-{} * {}), 0.0, 1.0)",
                self.float_text(*density),
                distance
            ),
            FogStyle::Linear { near, far, .. } => format!(
                "clamp(({} - {}) / ({} - {}), 0.0, 1.0)",
                distance,
                self.float_text(*near),
                self.float_text(*far),
                self.float_text(*near)
            ),
        };
        format!("mix({}, {}, {})", rgb, fog_color, factor)
    }

    fn broadcast(&self, text: &str, kind: NodeValueKind, target: NodeValueKind) -> String {
        if kind == target {
            text.to_string()
        } else {
            format!("{}({})", ctor(target, self.wgsl), text)
        }
    }

    fn uniform_text(&self, uniform: UniformInput) -> &'static str {
        if self.wgsl {
            match uniform {
                UniformInput::Time => "frame.resolution.z",
                UniformInput::Resolution => "frame.resolution.xy",
                UniformInput::CameraPosition => "frame.camera_position_frame.xyz",
                UniformInput::BaseColor => "material.base_color",
                UniformInput::Emissive => "material.emissive_cutoff.rgb",
                UniformInput::Metallic => "material.params.x",
                UniformInput::Roughness => "material.params.y",
                UniformInput::AlphaCutoff => "material.emissive_cutoff.a",
            }
        } else {
            match uniform {
                UniformInput::Time => "time",
                UniformInput::Resolution => "resolution",
                UniformInput::CameraPosition => "camera_position",
                UniformInput::BaseColor => "material_base_color",
                UniformInput::Emissive => "material_emissive",
                UniformInput::Metallic => "material_metallic",
                UniformInput::Roughness => "material_roughness",
                UniformInput::AlphaCutoff => "material_alpha_cutoff",
            }
        }
    }

    fn attribute_text(&self, attribute: VertexAttribute) -> &'static str {
        match self.scope {
            Scope::Vertex => match attribute {
                VertexAttribute::Position => "position",
                VertexAttribute::Normal => "normal",
                VertexAttribute::Uv => "uv",
                VertexAttribute::Tangent => "tangent",
                VertexAttribute::Color => "color",
            },
            Scope::Fragment => attribute.varying_name(),
            Scope::Post => "post_attributes_unavailable",
        }
    }

    fn view_expression(&self) -> String {
        if self.wgsl {
            "normalize(frame.camera_position_frame.xyz - v_world_position)".to_string()
        } else {
            "normalize(camera_position - v_world_position)".to_string()
        }
    }

    fn screen_uv_expression(&self) -> String {
        if self.wgsl {
            "frag_coord.xy / frame.resolution.xy".to_string()
        } else {
            "(gl_FragCoord.xy / resolution)".to_string()
        }
    }

    fn sample_material(&self, uv: &str) -> String {
        if self.wgsl {
            format!("textureSample(material_albedo, material_sampler, {})", uv)
        } else {
            format!(
                "texture(material_albedo, clamp({}, vec2(0.0), vec2(1.0)))",
                uv
            )
        }
    }

    fn post_param_text(&self, index: u8) -> String {
        let components = [
            ComponentIndex::X,
            ComponentIndex::Y,
            ComponentIndex::Z,
            ComponentIndex::W,
        ];
        let (block, index) = if index < 4 {
            ("values0", index)
        } else {
            ("values1", index - 4)
        };
        let letter = components
            .get(index as usize)
            .map_or('x', |component| component.letter());
        if self.wgsl {
            format!("params.{}.{}", block, letter)
        } else {
            format!("{}.{}", block, letter)
        }
    }

    // -- port resolution -----------------------------------------------------------

    /// Renders an output port expression, defaulting like the graph contract.
    pub(crate) fn port_text(&self, port: Option<&NodeInput>) -> String {
        match port {
            Some(NodeInput::Value(value)) => self.literal(value),
            Some(NodeInput::Node(id)) => self.name_of(*id),
            None => self.float_text(0.0),
        }
    }

    /// Renders the fragment color port expression.
    pub(crate) fn color_port_text(&self) -> String {
        match self.graph.fragment_color() {
            Some(NodeInput::Value(value)) => self.literal(value),
            Some(NodeInput::Node(id)) => self.name_of(*id),
            None => self.float_text(0.0),
        }
    }

    /// Renders the fragment alpha port; `None` falls back to `1.0`.
    pub(crate) fn alpha_port_text(&self) -> String {
        match self.graph.fragment_alpha() {
            Some(port) => self.port_text(Some(port)),
            None => self.float_text(1.0),
        }
    }
}

/// Vector constructor prefix for a value kind.
pub(crate) fn ctor(kind: NodeValueKind, wgsl: bool) -> String {
    if wgsl {
        format!("{}(", kind.wgsl_type())
    } else {
        format!("{}(", kind.glsl_type())
    }
}

/// Builds the deterministic CSE key of one node.
fn cse_key(node: &ShaderNodeData) -> Vec<u8> {
    let mut key = Vec::new();
    encode_kind_payload(&node.kind, &mut key);
    for input in &node.inputs {
        match input {
            NodeInput::Node(id) => {
                key.push(0);
                key.extend_from_slice(&id.0.to_le_bytes());
            }
            NodeInput::Value(value) => {
                key.push(1);
                for word in value.bit_pattern() {
                    key.extend_from_slice(&word.to_le_bytes());
                }
            }
        }
    }
    key
}

/// Resolves the value kind of a port from the resolved node table.
pub(crate) fn port_resolved_kind(
    resolved: &BTreeMap<ShaderNodeId, Option<NodeValueKind>>,
    port: Option<&NodeInput>,
    fallback: NodeValueKind,
) -> NodeValueKind {
    match port {
        Some(NodeInput::Node(id)) => resolved.get(id).copied().flatten().unwrap_or(fallback),
        Some(NodeInput::Value(value)) => value.kind(),
        None => fallback,
    }
}

/// Generates the WGSL module for a node material, matching the scenix
/// renderer bind-group contract (frame group 0, object group 1, material
/// group 2, lights group 3).
///
/// # Errors
/// Returns the first [`NodeDiagnostic`] produced by graph validation.
pub fn generate_material_wgsl(
    material: &NodeMaterial,
    options: &WgslOptions,
) -> Result<NodeWgslProgram, NodeDiagnostic> {
    let graph = material.graph();
    let resolved = graph.resolve(Stage::Fragment)?;
    let order = graph.fragment_order();

    let mut fs = Engine::new(true, Scope::Fragment, graph, &resolved);
    fs.compile_order(&order);
    let color_text = fs.color_port_text();
    let alpha_text = fs.alpha_port_text();
    let color_kind = port_resolved_kind(&resolved, graph.fragment_color(), NodeValueKind::Vec4);

    let mut vs = Engine::new(true, Scope::Vertex, graph, &resolved);
    let mut vertex_statements = String::new();
    let mut vertex_offset_text = String::new();
    if let Some(vertex_order) = graph.vertex_order() {
        vs.compile_order(&vertex_order);
        vertex_offset_text = vs.port_text(graph.vertex_offset());
        vertex_statements = vs.statements;
    }

    let mut source = String::with_capacity(2048);
    push_material_wgsl_header(&mut source, material, &fs.needs);
    push_material_wgsl_vertex(
        &mut source,
        options.vertex_entry,
        &vertex_statements,
        &vertex_offset_text,
    );
    push_material_wgsl_fragment(
        &mut source,
        options.fragment_entry,
        &fs.statements,
        &color_text,
        &alpha_text,
        color_kind,
    );

    Ok(NodeWgslProgram {
        label: Some(material.name().to_string()),
        source,
    })
}

/// Generates the WGSL module for a full-screen post node effect, matching
/// the scenix post-process binding contract (group 0: texture, sampler,
/// `PostParams` uniform).
///
/// # Errors
/// Returns the first [`NodeDiagnostic`] produced by graph validation.
pub fn generate_post_wgsl(
    effect: &PostNodeEffect,
    _options: &WgslOptions,
) -> Result<NodeWgslProgram, NodeDiagnostic> {
    let graph = effect.graph();
    let resolved = graph.resolve(Stage::Post)?;
    let order = graph.fragment_order();

    let mut fs = Engine::new(true, Scope::Post, graph, &resolved);
    fs.compile_order(&order);
    let color_text = fs.color_port_text();

    let mut source = String::with_capacity(1024);
    push_post_wgsl_header(&mut source, effect, &fs.needs);
    push_post_wgsl_vertex(&mut source);
    source.push_str(
        "@fragment\nfn fs_main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\n",
    );
    source.push_str(&fs.statements);
    let color_kind = port_resolved_kind(&resolved, graph.fragment_color(), NodeValueKind::Vec4);
    match color_kind {
        NodeValueKind::Vec3 => {
            source.push_str(&format!(
                "    return vec4<f32>({}, 1.0f);\n}}\n",
                color_text
            ));
        }
        _ => {
            source.push_str(&format!("    return vec4<f32>({});\n}}\n", color_text));
        }
    }

    Ok(NodeWgslProgram {
        label: Some(effect.name().to_string()),
        source,
    })
}

/// Conditional WG-syntax helper library, emitted before material and post
/// stages when their nodes are referenced.
pub(crate) fn push_wgsl_library_common(source: &mut String, needs: &Needs) {
    if needs.any_lighting() {
        source.push_str("fn scenix_light_lambert(base_color: vec3<f32>, n: vec3<f32>, world_position: vec3<f32>) -> vec3<f32> {\n");
        source.push_str("    let light_dir = normalize(-lights.directional_direction.xyz);\n");
        source.push_str(
            "    var diffuse = max(dot(n, light_dir), 0.0) * lights.directional_color.a;\n",
        );
        source.push_str("    let to_point = lights.point_position_range.xyz - world_position;\n");
        source.push_str("    let point_distance = max(length(to_point), 0.001);\n");
        source.push_str("    let point_range = lights.point_position_range.w;\n");
        source.push_str("    let point_attenuation = select(\n        1.0 / (point_distance * point_distance),\n        max(1.0 - point_distance / max(point_range, 0.001), 0.0),\n        point_range > 0.0,\n    );\n");
        source.push_str("    diffuse += max(dot(n, normalize(to_point)), 0.0) * lights.point_color.a * point_attenuation;\n");
        source.push_str("    let scene_ambient = lights.ambient.rgb + lights.environment.rgb * lights.environment.a;\n");
        source.push_str("    return base_color * (scene_ambient + vec3<f32>(diffuse));\n}\n");
    }
    if needs.phong {
        source.push_str("fn scenix_light_phong(base_color: vec3<f32>, n: vec3<f32>, world_position: vec3<f32>) -> vec3<f32> {\n");
        source.push_str("    let lit = scenix_light_lambert(base_color, n, world_position);\n");
        source.push_str("    let light_dir = normalize(-lights.directional_direction.xyz);\n");
        source.push_str(
            "    let view_dir = normalize(frame.camera_position_frame.xyz - world_position);\n",
        );
        source.push_str("    let half_dir = normalize(light_dir + view_dir);\n");
        source.push_str("    let roughness = max(material.params.y, 0.04);\n");
        source.push_str("    let spec_exp = 8.0 + (1.0 - roughness) * 88.0;\n");
        source.push_str(
            "    let specular = pow(max(dot(n, half_dir), 0.0), spec_exp) * (1.0 - roughness);\n",
        );
        source
            .push_str("    return lit + vec3<f32>(specular) * lights.directional_color.rgb;\n}\n");
    }
    if needs.toon {
        source.push_str("fn scenix_light_toon(base_color: vec3<f32>, n: vec3<f32>, world_position: vec3<f32>, bands: f32) -> vec3<f32> {\n");
        source.push_str("    let lit = scenix_light_lambert(base_color, n, world_position);\n");
        source.push_str("    let light_scale = max(lit, vec3<f32>(0.001));\n");
        source.push_str("    let quantized = floor(light_scale * bands) / (bands - 1.0);\n");
        source.push_str("    return clamp(quantized, vec3<f32>(0.0), vec3<f32>(1.0));\n}\n");
    }
    if needs.tonemap_reinhard {
        source.push_str("fn scenix_tonemap_reinhard(color: vec3<f32>) -> vec3<f32> {\n    return color / (vec3<f32>(1.0) + color);\n}\n");
    }
    if needs.tonemap_aces {
        source.push_str("fn scenix_tonemap_aces(color: vec3<f32>) -> vec3<f32> {\n");
        source.push_str("    let a = 2.51;\n    let b = 0.03;\n    let c = 2.43;\n    let d = 0.59;\n    let e = 0.14;\n");
        source.push_str("    return clamp((color * (a * color + vec3<f32>(b))) / (color * (c * color + vec3<f32>(d)) + vec3<f32>(e)), vec3<f32>(0.0), vec3<f32>(1.0));\n}\n");
    }
    if needs.tonemap_exposure {
        source.push_str("fn scenix_tonemap_exposure(color: vec3<f32>, exposure: f32) -> vec3<f32> {\n    return vec3<f32>(1.0) - exp(-color * vec3<f32>(exposure));\n}\n");
    }
    if needs.srgb_to_linear || needs.linear_to_srgb {
        source.push_str("fn scenix_srgb_channel_to_linear(channel: f32) -> f32 {\n");
        source.push_str("    let low = channel / 12.92;\n");
        source.push_str("    let high = pow((channel + 0.055) / 1.055, 2.4);\n");
        source.push_str("    return select(high, low, channel <= 0.04045);\n}\n");
        source.push_str("fn scenix_srgb_channel_from_linear(channel: f32) -> f32 {\n");
        source.push_str("    let low = max(channel, 0.0) * 12.92;\n");
        source.push_str("    let high = 1.055 * pow(max(channel, 0.0), 1.0 / 2.4) - 0.055;\n");
        source.push_str("    return select(high, low, channel <= 0.0031308);\n}\n");
    }
    if needs.srgb_to_linear {
        source.push_str("fn scenix_srgb_to_linear(color: vec3<f32>) -> vec3<f32> {\n");
        source.push_str("    return vec3<f32>(\n        scenix_srgb_channel_to_linear(color.r),\n        scenix_srgb_channel_to_linear(color.g),\n        scenix_srgb_channel_to_linear(color.b),\n    );\n}\n");
    }
    if needs.linear_to_srgb {
        source.push_str("fn scenix_linear_to_srgb(color: vec3<f32>) -> vec3<f32> {\n");
        source.push_str("    return vec3<f32>(\n        scenix_srgb_channel_from_linear(color.r),\n        scenix_srgb_channel_from_linear(color.g),\n        scenix_srgb_channel_from_linear(color.b),\n    );\n}\n");
    }
}

fn push_wgsl_library(source: &mut String, needs: &Needs) {
    push_wgsl_library_common(source, needs);
}

/// Material WGSL header: uniforms, bindings, and the conditional library.
fn push_material_wgsl_header(source: &mut String, material: &NodeMaterial, needs: &Needs) {
    source.push_str("// scenix v1.6 node material — generated by scenix-nodes\n");
    source.push_str(&format!("// material: {}\n\n", material.name()));

    source.push_str("struct FrameUniform {\n    view_projection: mat4x4<f32>,\n    camera_position_frame: vec4<f32>,\n    resolution: vec4<f32>,\n};\n\n");
    source.push_str("struct ObjectUniform {\n    world: mat4x4<f32>,\n};\n\n");
    source.push_str("struct MaterialUniform {\n    base_color: vec4<f32>,\n    emissive_cutoff: vec4<f32>,\n    params: vec4<f32>,\n};\n\n");
    source.push_str("struct LightUniform {\n    ambient: vec4<f32>,\n    directional_direction: vec4<f32>,\n    directional_color: vec4<f32>,\n    point_position_range: vec4<f32>,\n    point_color: vec4<f32>,\n    spot_direction_angle: vec4<f32>,\n    spot_color: vec4<f32>,\n    environment: vec4<f32>,\n    counts: vec4<f32>,\n};\n\n");

    source.push_str("struct VsOut {\n    @builtin(position) position: vec4<f32>,\n    @location(0) color: vec4<f32>,\n    @location(1) normal: vec3<f32>,\n    @location(2) uv: vec2<f32>,\n    @location(3) world_position: vec3<f32>,\n    @location(4) tangent: vec4<f32>,\n    @location(5) local: vec3<f32>,\n};\n\n");

    source.push_str("@group(0) @binding(0) var<uniform> frame: FrameUniform;\n");
    source.push_str("@group(1) @binding(0) var<uniform> object: ObjectUniform;\n");
    source.push_str("@group(2) @binding(0) var<uniform> material: MaterialUniform;\n");
    source.push_str("@group(2) @binding(1) var material_sampler: sampler;\n");
    source.push_str("@group(2) @binding(2) var material_albedo: texture_2d<f32>;\n");
    source.push_str("@group(3) @binding(0) var<uniform> lights: LightUniform;\n\n");

    push_wgsl_library(source, needs);
    source.push('\n');
}

/// Material WGSL vertex stage.
fn push_material_wgsl_vertex(
    source: &mut String,
    entry: &'static str,
    statements: &str,
    offset_text: &str,
) {
    source.push_str(&format!("@vertex\nfn {}(", entry));
    source.push_str("\n    @location(0) position: vec3<f32>,");
    source.push_str("\n    @location(1) normal: vec3<f32>,");
    source.push_str("\n    @location(2) uv: vec2<f32>,");
    source.push_str("\n    @location(3) color: vec4<f32>,");
    source.push_str("\n    @location(4) tangent: vec4<f32>,");
    source.push_str("\n) -> VsOut {\n");
    source.push_str("    var displaced = position;\n");
    source.push_str(statements);
    if !offset_text.is_empty() {
        source.push_str(&format!("    displaced = displaced + {};\n", offset_text));
    }
    source.push_str("    let vs_world4 = object.world * vec4<f32>(displaced, 1.0);\n");
    source.push_str("    var out: VsOut;\n");
    source.push_str("    out.position = frame.view_projection * vs_world4;\n");
    source.push_str("    out.color = color;\n");
    source.push_str("    out.normal = normalize((object.world * vec4<f32>(normal, 0.0)).xyz);\n");
    source.push_str("    out.uv = uv;\n");
    source.push_str("    out.world_position = vs_world4.xyz;\n");
    source.push_str("    out.tangent = tangent;\n");
    source.push_str("    out.local = displaced;\n    return out;\n}\n\n");
}

/// Material WGSL fragment stage.
fn push_material_wgsl_fragment(
    source: &mut String,
    entry: &'static str,
    statements: &str,
    color_text: &str,
    alpha_text: &str,
    color_kind: NodeValueKind,
) {
    source.push_str(&format!("@fragment\nfn {}(", entry));
    source.push_str("\n    @location(0) v_color: vec4<f32>,");
    source.push_str("\n    @location(1) v_normal: vec3<f32>,");
    source.push_str("\n    @location(2) v_uv: vec2<f32>,");
    source.push_str("\n    @location(3) v_world_position: vec3<f32>,");
    source.push_str("\n    @location(4) v_tangent: vec4<f32>,");
    source.push_str("\n    @location(5) v_position: vec3<f32>,");
    source.push_str("\n    @builtin(position) frag_coord: vec4<f32>,");
    source.push_str("\n) -> @location(0) vec4<f32> {\n");
    source.push_str(statements);
    match color_kind {
        // A vec3 lighting/fog result carries no alpha; the explicit port
        // text fills it.
        NodeValueKind::Vec3 => {
            source.push_str(&format!(
                "    var final_color = vec4<f32>({color}, {alpha});\n",
                color = color_text,
                alpha = alpha_text
            ));
        }
        _ => {
            source.push_str(&format!(
                "    var final_color = vec4<f32>({});\n    final_color.a = {};\n",
                color_text, alpha_text
            ));
        }
    }
    source.push_str("    if (material.emissive_cutoff.a >= 0.0 && final_color.a < material.emissive_cutoff.a) {\n        discard;\n    }\n");
    source.push_str("    return final_color;\n}\n");
}

/// Post WGSL header: bindings and the conditional library.
fn push_post_wgsl_header(source: &mut String, effect: &PostNodeEffect, needs: &Needs) {
    source.push_str("// scenix v1.6 post node effect — generated by scenix-nodes\n");
    source.push_str(&format!("// effect: {}\n\n", effect.name()));
    source.push_str("// Full-screen pass: group(0) binding(0) texture, binding(1) sampler,\n// binding(2) PostParams uniform.\n\n");
    source.push_str("struct VertexOut {\n    @builtin(position) position: vec4<f32>,\n    @location(0) uv: vec2<f32>,\n};\n\n");
    source
        .push_str("struct PostParams {\n    values0: vec4<f32>,\n    values1: vec4<f32>,\n};\n\n");
    source.push_str("@group(0) @binding(0) var post_texture: texture_2d<f32>;\n");
    source.push_str("@group(0) @binding(1) var post_sampler: sampler;\n");
    source.push_str("@group(0) @binding(2) var<uniform> params: PostParams;\n\n");
    push_wgsl_library(source, needs);
    source.push('\n');
}

/// Post WGSL full-screen triangle vertex stage.
fn push_post_wgsl_vertex(source: &mut String) {
    source
        .push_str("@vertex\nfn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {\n");
    source.push_str("    var positions = array<vec2<f32>, 3>(\n        vec2<f32>(-1.0, -3.0),\n        vec2<f32>(3.0, 1.0),\n        vec2<f32>(-1.0, 1.0),\n    );\n");
    source.push_str("    let position = positions[vertex_index];\n    var out: VertexOut;\n");
    source.push_str("    out.position = vec4<f32>(position, 0.0, 1.0);\n");
    source.push_str("    out.uv = position * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);\n    return out;\n}\n\n");
}
