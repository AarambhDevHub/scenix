use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::graph::{NodeDiagnostic, ShaderGraph, ShaderNodeId};
use crate::node::{NodeInput, ShaderNodeKind, Stage, ToneMapKind};
use crate::value::{NodeValue, NodeValueKind};

/// Post parameter slots consumed by the bundled presets and exposed by the
/// WGSL/GLSL backends as `values0`/`values1` (`[f32; 8]` total). Hosts drive
/// these knobs per frame through the post parameter uniform.
pub mod slots {
    /// Vignette strength (`0`..`1`, higher darkens more).
    pub const VIGNETTE_STRENGTH: u8 = 0;
    /// Desaturation amount (`0`..`1`).
    pub const DESATURATE_AMOUNT: u8 = 0;
    /// Posterize level count (integer `>= 2`).
    pub const POSTERIZE_LEVELS: u8 = 0;
    /// Pixelation grid size (integer `>= 2`).
    pub const PIXELATE_GRID: u8 = 0;
    /// Color-grade saturation multiplier.
    pub const GRADE_SATURATION: u8 = 0;
    /// Color-grade brightness multiplier.
    pub const GRADE_BRIGHTNESS: u8 = 1;
    /// Color-grade contrast multiplier (1.0 = unchanged).
    pub const GRADE_CONTRAST: u8 = 2;
}

/// A full-screen node graph bound to the scenix post pass contract: the
/// source texture is sampled at `v_uv`, dynamic tuning arrives through the
/// 8-float `PostParam` block, and the output color is returned as `vec4`.
///
/// Every bundled preset validates for [`Stage::Post`]; custom graphs do the
/// same via [`PostNodeEffect::validate`]. Unlike material graphs, post
/// graphs cannot read scene uniforms or material textures — dynamic values
/// come through `PostParam` slots, which the host fills every frame.
///
/// # Example
/// ```
/// use scenix_nodes::{slots, PostNodeEffect};
///
/// let mut effect = PostNodeEffect::vignette();
/// assert!(effect.validate().is_empty());
/// assert_eq!(effect.name(), "vignette");
/// // Effect strength is driven at draw time through slot 0:
/// let strength = slots::VIGNETTE_STRENGTH; // index 0 of the [f32; 8] block
/// assert!(strength < 8);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PostNodeEffect {
    name: String,
    graph: ShaderGraph,
}

impl PostNodeEffect {
    /// Creates a post effect from a validated graph.
    #[must_use]
    pub fn new(name: impl Into<String>, graph: ShaderGraph) -> Self {
        Self {
            name: name.into(),
            graph,
        }
    }

    /// Sets the effect name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Returns the effect name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the graph.
    #[inline]
    pub fn graph(&self) -> &ShaderGraph {
        &self.graph
    }

    /// Returns the mutable graph (re-validate after edits).
    #[inline]
    pub fn graph_mut(&mut self) -> &mut ShaderGraph {
        &mut self.graph
    }

    /// Validates the graph for the post stage.
    pub fn validate(&self) -> Vec<NodeDiagnostic> {
        self.graph.validate(Stage::Post)
    }

    /// Identity effect: the source frame passes through untouched.
    #[must_use]
    pub fn passthrough() -> Self {
        let mut graph = ShaderGraph::new();
        let source = base_source(&mut graph);
        graph.add_fragment_output(source, None);
        Self::new("passthrough", graph)
    }

    /// Radial darkening. `PostParam(0)` is the strength in `0..=1`.
    #[must_use]
    pub fn vignette() -> Self {
        let mut graph = ShaderGraph::new();
        let uv = graph.push(ShaderNodeKind::PostUv);
        let centered = graph.add(
            ShaderNodeKind::Subtract,
            vec![node(uv), value(NodeValue::splat(NodeValueKind::Vec2, 0.5))],
        );
        let radius = graph.chain(ShaderNodeKind::Length, centered);
        let strength = graph.push(ShaderNodeKind::PostParam(slots::VIGNETTE_STRENGTH));
        let bell = graph.add(
            ShaderNodeKind::Smoothstep,
            vec![
                value(NodeValue::float(0.35)),
                value(NodeValue::float(0.9)),
                node(radius),
            ],
        );
        let falloff = graph.chain(ShaderNodeKind::OneMinus, bell);
        let gain = graph.add(
            ShaderNodeKind::Multiply,
            vec![node(falloff), node(strength)],
        );
        let clamped = graph.chain(ShaderNodeKind::Saturate, gain);
        let source = graph.chain(ShaderNodeKind::PostSource, uv);
        let output = graph.add(ShaderNodeKind::Multiply, vec![node(source), node(clamped)]);
        graph.add_fragment_output(output, None);
        Self::new("vignette", graph)
    }

    /// Luma-preserved desaturation. `PostParam(0)` is the amount in `0..=1`.
    #[must_use]
    pub fn desaturate() -> Self {
        let mut graph = ShaderGraph::new();
        let source = base_source(&mut graph);
        let luma = graph.add(
            ShaderNodeKind::Dot,
            vec![
                node(source),
                value(NodeValue::vec4(scenix_math::Vec4::new(
                    0.2126, 0.7152, 0.0722, 0.0,
                ))),
            ],
        );
        let gray = graph.chain(ShaderNodeKind::Splat(NodeValueKind::Vec4), luma);
        let amount = graph.push(ShaderNodeKind::PostParam(slots::DESATURATE_AMOUNT));
        let output = graph.add(
            ShaderNodeKind::Mix,
            vec![node(source), node(gray), node(amount)],
        );
        graph.add_fragment_output(output, None);
        Self::new("desaturate", graph)
    }

    /// Quantized color bands. `PostParam(0)` is the level count `>= 2`.
    #[must_use]
    pub fn posterize() -> Self {
        let mut graph = ShaderGraph::new();
        let source = base_source(&mut graph);
        let levels = graph.push(ShaderNodeKind::PostParam(slots::POSTERIZE_LEVELS));
        let scaled = graph.add(ShaderNodeKind::Multiply, vec![node(source), node(levels)]);
        let floored = graph.chain(ShaderNodeKind::Floor, scaled);
        let quantized = graph.add(ShaderNodeKind::Divide, vec![node(floored), node(levels)]);
        let output = graph.chain(ShaderNodeKind::Saturate, quantized);
        graph.add_fragment_output(output, None);
        Self::new("posterize", graph)
    }

    /// Blocky pixelation. `PostParam(0)` is the grid size `>= 2`.
    #[must_use]
    pub fn pixelate() -> Self {
        let mut graph = ShaderGraph::new();
        let uv = graph.push(ShaderNodeKind::PostUv);
        let grid = graph.push(ShaderNodeKind::PostParam(slots::PIXELATE_GRID));
        let scaled = graph.add(ShaderNodeKind::Multiply, vec![node(uv), node(grid)]);
        let floored = graph.chain(ShaderNodeKind::Floor, scaled);
        let snapped = graph.add(ShaderNodeKind::Divide, vec![node(floored), node(grid)]);
        let source = graph.chain(ShaderNodeKind::PostSource, snapped);
        graph.add_fragment_output(source, None);
        Self::new("pixelate", graph)
    }

    /// Saturation, brightness, and contrast grade:
    /// `PostParam(0)` saturation, `PostParam(1)` brightness, `PostParam(2)`
    /// contrast (all multipliers, 1.0 = unchanged).
    #[must_use]
    pub fn color_grade() -> Self {
        let mut graph = ShaderGraph::new();
        let source = base_source(&mut graph);
        let saturation = graph.push(ShaderNodeKind::PostParam(slots::GRADE_SATURATION));
        let brightness = graph.push(ShaderNodeKind::PostParam(slots::GRADE_BRIGHTNESS));
        let contrast = graph.push(ShaderNodeKind::PostParam(slots::GRADE_CONTRAST));
        let luma = graph.add(
            ShaderNodeKind::Dot,
            vec![
                node(source),
                value(NodeValue::vec4(scenix_math::Vec4::new(
                    0.2126, 0.7152, 0.0722, 0.0,
                ))),
            ],
        );
        let gray = graph.chain(ShaderNodeKind::Splat(NodeValueKind::Vec4), luma);
        let desaturated = graph.add(
            ShaderNodeKind::Mix,
            vec![node(source), node(gray), node(saturation)],
        );
        let brightened = graph.add(
            ShaderNodeKind::Multiply,
            vec![node(desaturated), node(brightness)],
        );
        let shifted = graph.add(
            ShaderNodeKind::Subtract,
            vec![
                node(brightened),
                value(NodeValue::vec4(scenix_math::Vec4::new(0.5, 0.5, 0.5, 0.5))),
            ],
        );
        let weighted = graph.add(
            ShaderNodeKind::Multiply,
            vec![node(shifted), node(contrast)],
        );
        let output = graph.add(
            ShaderNodeKind::Add,
            vec![
                node(weighted),
                value(NodeValue::vec4(scenix_math::Vec4::new(0.5, 0.5, 0.5, 0.5))),
            ],
        );
        graph.add_fragment_output(output, None);
        Self::new("color_grade", graph)
    }

    /// Baked exposure tone map; the factor is part of the graph identity.
    #[must_use]
    pub fn exposure(exposure: f32) -> Self {
        let mut graph = ShaderGraph::new();
        let source = base_source(&mut graph);
        let output = graph.chain(
            ShaderNodeKind::ToneMap(ToneMapKind::Exposure(exposure)),
            source,
        );
        graph.add_fragment_output(output, None);
        Self::new("exposure", graph)
    }
}

/// Samples the post source through `PostSource(PostUv)`.
fn base_source(graph: &mut ShaderGraph) -> ShaderNodeId {
    let uv = graph.push(ShaderNodeKind::PostUv);
    graph.chain(ShaderNodeKind::PostSource, uv)
}

/// Wraps a node id as a node input.
fn node(id: ShaderNodeId) -> NodeInput {
    NodeInput::Node(id)
}

/// Builds an inline constant input.
fn value(value: NodeValue) -> NodeInput {
    NodeInput::Value(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::UniformInput;

    #[test]
    fn presets_validate_for_post_stage() {
        let effects = vec![
            PostNodeEffect::passthrough(),
            PostNodeEffect::vignette(),
            PostNodeEffect::desaturate(),
            PostNodeEffect::posterize(),
            PostNodeEffect::pixelate(),
            PostNodeEffect::color_grade(),
            PostNodeEffect::exposure(1.25),
        ];
        for effect in effects {
            assert!(
                effect.validate().is_empty(),
                "{} graph rejected: {:?}",
                effect.name(),
                effect.validate()
            );
        }
    }

    #[test]
    fn uniforms_are_rejected_in_post_graphs() {
        let mut graph = ShaderGraph::new();
        let time = graph.push(ShaderNodeKind::Uniform(UniformInput::Time));
        graph.add_fragment_output(time, None);
        let effect = PostNodeEffect::new("uniform_in_post", graph);
        assert!(!effect.validate().is_empty());
    }
}
