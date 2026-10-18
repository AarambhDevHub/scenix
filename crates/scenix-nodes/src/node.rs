use alloc::vec::Vec;
use core::fmt;

use crate::value::{NodeValue, NodeValueKind};

/// Graph-local handle to one node's output.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShaderNodeId(pub u32);

impl fmt::Display for ShaderNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node{}", self.0)
    }
}

/// One input port of a node: either inline data or another node's output.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeInput {
    /// Reads the output of another node.
    Node(ShaderNodeId),
    /// Inline constant (participates in common-subexpression elimination).
    Value(NodeValue),
}

impl From<ShaderNodeId> for NodeInput {
    #[inline]
    fn from(id: ShaderNodeId) -> Self {
        Self::Node(id)
    }
}

impl From<NodeValue> for NodeInput {
    #[inline]
    fn from(value: NodeValue) -> Self {
        Self::Value(value)
    }
}

impl From<f32> for NodeInput {
    #[inline]
    fn from(value: f32) -> Self {
        Self::Value(NodeValue::float(value))
    }
}

/// Renderable attribute exposed to the fragment stage through fixed varyings.
///
/// Attributes live in the vertex stage but are interpolated for every graph
/// that reads them, matching the scenix renderer's mesh layout (`position`,
/// `normal`, `uv`, vertex `color`, and `tangent`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VertexAttribute {
    /// Local-space vertex position.
    #[default]
    Position,
    /// Local-space vertex normal.
    Normal,
    /// Primary texture coordinate.
    Uv,
    /// Tangent vector with handedness in `w`.
    Tangent,
    /// Per-vertex color.
    Color,
}

impl VertexAttribute {
    /// Returns the value type interpolated to the fragment stage.
    #[inline]
    pub const fn value_kind(self) -> NodeValueKind {
        match self {
            Self::Position | Self::Normal => NodeValueKind::Vec3,
            Self::Uv => NodeValueKind::Vec2,
            Self::Tangent | Self::Color => NodeValueKind::Vec4,
        }
    }

    /// Returns the fixed varying identifier used by both backends.
    #[inline]
    pub const fn varying_name(self) -> &'static str {
        match self {
            Self::Position => "v_position",
            Self::Normal => "v_normal",
            Self::Uv => "v_uv",
            Self::Tangent => "v_tangent",
            Self::Color => "v_color",
        }
    }
}

/// Read-only live CPU state exposed to node graphs as uniforms.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UniformInput {
    /// Elapsed seconds since the program start.
    #[default]
    Time,
    /// Render resolution in pixels (`x`, `y`).
    Resolution,
    /// World-space camera position.
    CameraPosition,
    /// Live material base color (linear RGBA).
    BaseColor,
    /// Live material emissive color (linear RGB).
    Emissive,
    /// Live material metallic factor.
    Metallic,
    /// Live material roughness factor.
    Roughness,
    /// Alpha-test cutoff; negative when alpha testing is disabled.
    AlphaCutoff,
}

impl UniformInput {
    /// Returns the uniform value type.
    #[inline]
    pub const fn value_kind(self) -> NodeValueKind {
        match self {
            Self::Time | Self::Metallic | Self::Roughness | Self::AlphaCutoff => {
                NodeValueKind::Float
            }
            Self::Resolution => NodeValueKind::Vec2,
            Self::CameraPosition | Self::Emissive => NodeValueKind::Vec3,
            Self::BaseColor => NodeValueKind::Vec4,
        }
    }

    /// Returns the canonical activation name used by both backends.
    #[inline]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Time => "time",
            Self::Resolution => "resolution",
            Self::CameraPosition => "camera_position",
            Self::BaseColor => "material_base_color",
            Self::Emissive => "material_emissive",
            Self::Metallic => "material_metallic",
            Self::Roughness => "material_roughness",
            Self::AlphaCutoff => "material_alpha_cutoff",
        }
    }

    /// Returns whether this uniform is available in post-processing graphs.
    #[inline]
    pub const fn available_in_post(self) -> bool {
        matches!(self, Self::Time | Self::Resolution)
    }
}

/// Surface shading evaluation used by the `Lighting` node.
///
/// Lighting is evaluated with the scene lights uploaded by the renderer
/// (ambient + directional + point contribution, matching the built-in v1
/// preview shading) and is only available in the fragment stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LightingStyle {
    /// Wrapped diffuse shading with ambient and point contribution.
    #[default]
    Lambert,
    /// Diffuse shading plus Blinn-Phong specular driven by material roughness.
    Phong,
    /// Discrete cel bands; the parameter is the band count (`>= 2`).
    Toon(u32),
}

impl LightingStyle {
    /// Validates the band parameter at construction time.
    #[inline]
    pub const fn is_valid(self) -> bool {
        match self {
            Self::Lambert | Self::Phong => true,
            Self::Toon(bands) => bands >= 2,
        }
    }

    /// Returns the canonical activation name used by both backends.
    #[inline]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Lambert => "scenix_light_lambert",
            Self::Phong => "scenix_light_phong",
            Self::Toon(_) => "scenix_light_toon",
        }
    }
}

/// Tone map operator available in both material and post graphs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ToneMapKind {
    /// Leaves color unchanged.
    #[default]
    None,
    /// Reinhard operator `c / (1 + c)`.
    Reinhard,
    /// ACES-inspired filmic curve.
    Aces,
    /// Exponential exposure curve with the stored exposure.
    Exposure(f32),
}

impl ToneMapKind {
    /// Returns the activation name used by both backends.
    #[inline]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::None => "scenix_tonemap_none",
            Self::Reinhard => "scenix_tonemap_reinhard",
            Self::Aces => "scenix_tonemap_aces",
            Self::Exposure(_) => "scenix_tonemap_exposure",
        }
    }
}

/// Fog evaluation used by the `Fog` node (fragment stage only).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FogStyle {
    /// Exponential density fog toward `color`.
    Exponential {
        /// Fog density factor.
        density: f32,
        /// Fog color in linear RGB.
        color: NodeValue,
    },
    /// Linear distance-range fog toward `color`.
    Linear {
        /// Near distance.
        near: f32,
        /// Far distance.
        far: f32,
        /// Fog color in linear RGB.
        color: NodeValue,
    },
}

/// Signed channel selector for the `Component` node.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ComponentIndex {
    /// `.x` channel.
    #[default]
    X,
    /// `.y` channel.
    Y,
    /// `.z` channel.
    Z,
    /// `.w` channel.
    W,
}

impl ComponentIndex {
    /// Returns the zero-based channel index.
    #[inline]
    pub const fn index(self) -> u8 {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
            Self::W => 3,
        }
    }

    /// Returns the swizzle letter used by both backends.
    #[inline]
    pub const fn letter(self) -> char {
        match self {
            Self::X => 'x',
            Self::Y => 'y',
            Self::Z => 'z',
            Self::W => 'w',
        }
    }
}

/// Per-stage availability of a node kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StageKind {
    /// Usable in the material fragment stage only.
    FragmentOnly,
    /// Usable in both material vertex and fragment stages.
    VertexFragment,
    /// Usable in the material fragment stage and the post stage.
    FragmentAndPost,
    /// Usable in the post-processing stage only.
    PostOnly,
    /// Usable everywhere, including post graphs.
    Any,
}

impl StageKind {
    /// Returns whether the kind is usable while lowering `stage`.
    #[inline]
    pub const fn allows(self, stage: Stage) -> bool {
        match stage {
            Stage::Vertex => matches!(self, Self::VertexFragment | Self::Any),
            Stage::Fragment => matches!(
                self,
                Self::VertexFragment | Self::FragmentAndPost | Self::FragmentOnly | Self::Any
            ),
            Stage::Post => matches!(self, Self::FragmentAndPost | Self::PostOnly | Self::Any),
        }
    }
}

/// Compilation stage a graph is validated and lowered for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Stage {
    /// Material vertex stage (displacement expressions).
    Vertex,
    /// Material fragment stage.
    #[default]
    Fragment,
    /// Full-screen post-processing stage.
    Post,
}

/// One node kind: the operation plus any inline parameters.
///
/// Kinds with float payloads compare through bit-exact patterns so code
/// generators can merge nodes deterministically.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ShaderNodeKind {
    // -- data -----------------------------------------------------------------
    /// Inline constant.
    Constant(NodeValue),
    /// Live CPU uniform described by [`UniformInput`].
    Uniform(UniformInput),
    /// Interpolated mesh attribute ([`VertexAttribute`]).
    Attribute(VertexAttribute),
    /// Interpolated world-space position.
    WorldPosition,
    /// Interpolated world-space unit normal.
    WorldNormal,
    /// Unit direction from the surface to the camera.
    ViewDirection,
    /// Screen UV in `0..=1` derived from fragment coordinates.
    ScreenUv,
    /// Reads entry `index` (`0..=7`) of the post parameter block.
    PostParam(u8),

    /// Samples material texture slot 0 with the `uv` input.
    TextureSample,
    /// Broadcasts a float input into `kind`.
    Splat(NodeValueKind),
    /// Extracts channel `index` from a vec2/3/4 input.
    Component(ComponentIndex),

    // -- math -----------------------------------------------------------------
    /// `a + b` (scalar broadcast allowed).
    Add,
    /// `a - b` (scalar broadcast allowed).
    Subtract,
    /// `a * b` (scalar broadcast allowed).
    Multiply,
    /// `a / b` (scalar broadcast allowed).
    Divide,
    /// `min(a, b)` (scalar broadcast allowed).
    Min,
    /// `max(a, b)` (scalar broadcast allowed).
    Max,
    /// `dot(a, b)`; both inputs must match in `Vec2`..=`Vec4`.
    Dot,
    /// `cross(a, b)`; both inputs are `Vec3`.
    Cross,
    /// `length(v)`; the input is `Vec2`..=`Vec4`.
    Length,
    /// `normalize(v)`.
    Normalize,
    /// `mix(a, b, t)` where `t` is a float in `0..=1`.
    Mix,
    /// `clamp(x, lo, hi)`; all inputs share one type.
    Clamp,
    /// `clamp(x, 0, 1)`.
    Saturate,
    /// `abs(x)`.
    Abs,
    /// `pow(x, exponent)`; both inputs are floats.
    Power,
    /// `sin(x)`.
    Sin,
    /// `cos(x)`.
    Cos,
    /// `1 - x` (scalar broadcast allowed).
    OneMinus,
    /// `fract(x)`.
    Fract,
    /// `floor(x)`.
    Floor,
    /// `step(edge, x)`; both inputs are floats.
    Step,
    /// `smoothstep(edge0, edge1, x)`; all inputs are floats.
    Smoothstep,
    /// Linear remap of `x` from one range into another; all inputs are floats.
    /// Inputs: `x`, `from_min`, `from_max`, `to_min`, `to_max`.
    Remap,
    /// Fresnel rim factor `pow(1 - |dot(n, view_dir)|, power)`.
    Fresnel(f32),

    // -- color and lighting -----------------------------------------------------
    /// sRGB → linear conversion; input is `Vec3`.
    SrgbToLinear,
    /// Linear → sRGB conversion; input is `Vec3`.
    LinearToSrgb,
    /// Scene lighting evaluation ([`LightingStyle`]).
    Lighting(LightingStyle),
    /// Tone-mapping evaluation ([`ToneMapKind`]).
    ToneMap(ToneMapKind),
    /// Fog evaluation ([`FogStyle`]).
    Fog(FogStyle),

    // -- post-processing stage ------------------------------------------------------
    /// Samples the post source texture at the `uv` input.
    PostSource,
    /// Interpolated full-screen pass UV in `0..=1`.
    PostUv,
}

impl ShaderNodeKind {
    /// Number of fixed input ports for this kind.
    #[inline]
    pub const fn port_count(&self) -> usize {
        match self {
            Self::Constant(_)
            | Self::Uniform(_)
            | Self::Attribute(_)
            | Self::WorldPosition
            | Self::WorldNormal
            | Self::ViewDirection
            | Self::ScreenUv
            | Self::PostParam(_) => 0,
            Self::PostUv => 0,
            Self::TextureSample
            | Self::Splat(_)
            | Self::Component(_)
            | Self::Length
            | Self::Normalize
            | Self::Saturate
            | Self::Abs
            | Self::Sin
            | Self::Cos
            | Self::OneMinus
            | Self::Fract
            | Self::Floor
            | Self::Fresnel(_)
            | Self::SrgbToLinear
            | Self::LinearToSrgb
            | Self::Lighting(_)
            | Self::ToneMap(_)
            | Self::Fog(_)
            | Self::PostSource => 1,
            Self::Add
            | Self::Subtract
            | Self::Multiply
            | Self::Divide
            | Self::Min
            | Self::Max
            | Self::Dot
            | Self::Cross
            | Self::Power
            | Self::Step => 2,
            Self::Mix | Self::Clamp | Self::Smoothstep => 3,
            Self::Remap => 5,
        }
    }

    /// Output value type given the resolved input types.
    pub fn output_kind(&self, inputs: &[NodeValueKind]) -> NodeValueKind {
        match self {
            Self::Constant(value) => value.kind(),
            Self::Uniform(uniform) => uniform.value_kind(),
            Self::Attribute(attribute) => attribute.value_kind(),
            Self::WorldPosition | Self::WorldNormal | Self::ViewDirection => NodeValueKind::Vec3,
            Self::ScreenUv | Self::PostUv => NodeValueKind::Vec2,
            Self::PostParam(_) => NodeValueKind::Float,
            Self::TextureSample | Self::PostSource => NodeValueKind::Vec4,
            Self::Splat(kind) => *kind,
            Self::Component(_) => NodeValueKind::Float,
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide | Self::Min | Self::Max => {
                broadcast_kind(inputs[0], inputs[1])
            }
            Self::Dot | Self::Length => NodeValueKind::Float,
            Self::Cross => NodeValueKind::Vec3,
            Self::Normalize => inputs[0],
            Self::Mix => inputs[0],
            Self::Clamp
            | Self::Saturate
            | Self::Abs
            | Self::OneMinus
            | Self::Fract
            | Self::Floor => inputs[0],
            Self::Power | Self::Sin | Self::Cos | Self::Step | Self::Smoothstep | Self::Remap => {
                NodeValueKind::Float
            }
            Self::Fresnel(_) => NodeValueKind::Float,
            Self::SrgbToLinear | Self::LinearToSrgb | Self::Lighting(_) | Self::ToneMap(_) => {
                NodeValueKind::Vec3
            }
            Self::Fog(_) => NodeValueKind::Vec3,
        }
    }

    /// Stage availability for this kind.
    #[inline]
    pub const fn stage_kind(&self) -> StageKind {
        match self {
            Self::Constant(_) => StageKind::Any,
            // Uniforms ride the material UBOs and attributes differ between
            // vertex layout and fragment varyings — neither is bound in the
            // post stage.
            Self::Uniform(_) | Self::Attribute(_) => StageKind::VertexFragment,
            Self::PostSource | Self::PostParam(_) | Self::PostUv => StageKind::PostOnly,
            Self::WorldPosition
            | Self::WorldNormal
            | Self::ViewDirection
            | Self::ScreenUv
            | Self::TextureSample
            | Self::Fresnel(_)
            | Self::SrgbToLinear
            | Self::LinearToSrgb
            | Self::Lighting(_)
            | Self::Fog(_) => StageKind::FragmentOnly,
            Self::ToneMap(_) => StageKind::FragmentAndPost,
            _ => StageKind::Any,
        }
    }

    /// Per-port pinned input type expectation; `None` means flexibly typed.
    #[inline]
    pub const fn input_expectation(&self, slot: usize) -> Option<NodeValueKind> {
        match self {
            Self::TextureSample | Self::PostSource if slot == 0 => Some(NodeValueKind::Vec2),
            Self::Cross | Self::Fresnel(_) => Some(NodeValueKind::Vec3),
            Self::Mix if slot == 2 => Some(NodeValueKind::Float),
            Self::Power | Self::Step | Self::Smoothstep | Self::Remap => Some(NodeValueKind::Float),
            _ => None,
        }
    }

    /// Returns the canonical op identifier rendered by both backends.
    pub const fn identifier(&self) -> &'static str {
        match self {
            Self::Constant(_) => "constant",
            Self::Uniform(_) => "uniform",
            Self::Attribute(_) => "attribute",
            Self::WorldPosition => "world_position",
            Self::WorldNormal => "world_normal",
            Self::ViewDirection => "view_direction",
            Self::ScreenUv => "screen_uv",
            Self::TextureSample => "texture_sample",
            Self::Splat(_) => "splat",
            Self::Component(_) => "component",
            Self::Add => "add",
            Self::Subtract => "subtract",
            Self::Multiply => "multiply",
            Self::Divide => "divide",
            Self::Min => "min",
            Self::Max => "max",
            Self::Dot => "dot",
            Self::Cross => "cross",
            Self::Length => "length",
            Self::Normalize => "normalize",
            Self::Mix => "mix",
            Self::Clamp => "clamp",
            Self::Saturate => "saturate",
            Self::Abs => "abs",
            Self::Power => "power",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::OneMinus => "one_minus",
            Self::Fract => "fract",
            Self::Floor => "floor",
            Self::Step => "step",
            Self::Smoothstep => "smoothstep",
            Self::Remap => "remap",
            Self::Fresnel(_) => "fresnel",
            Self::SrgbToLinear => "srgb_to_linear",
            Self::LinearToSrgb => "linear_to_srgb",
            Self::Lighting(_) => "lighting",
            Self::ToneMap(_) => "tonemap",
            Self::Fog(_) => "fog",
            Self::PostSource => "post_source",
            Self::PostParam(_) => "post_param",
            Self::PostUv => "post_uv",
        }
    }
}

/// Broadcasting resolution used by arithmetic nodes: matching types pass
/// through, otherwise a float operand broadcasts to the other side.
#[inline]
pub(crate) fn broadcast_kind(a: NodeValueKind, b: NodeValueKind) -> NodeValueKind {
    if a == b || !a.is_float() { a } else { b }
}

/// One graph node: kind plus resolved inputs.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShaderNodeData {
    /// Destination handle.
    pub id: ShaderNodeId,
    /// Operation and inline parameters.
    pub kind: ShaderNodeKind,
    /// Inputs (one per port; shorter lists leave trailing ports unset).
    pub inputs: Vec<NodeInput>,
}

impl ShaderNodeData {
    /// Creates a node from a kind and input list.
    #[inline]
    pub fn new(id: ShaderNodeId, kind: ShaderNodeKind, inputs: Vec<NodeInput>) -> Self {
        Self { id, kind, inputs }
    }

    /// Returns the input at `slot`, if set.
    #[inline]
    pub fn input(&self, slot: usize) -> Option<&NodeInput> {
        self.inputs.get(slot)
    }
}

impl fmt::Display for ShaderNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.id, self.kind.identifier())
    }
}
