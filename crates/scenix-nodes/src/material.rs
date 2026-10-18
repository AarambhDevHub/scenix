use alloc::string::String;
use alloc::vec::Vec;
use scenix_core::{Color, TextureId};
use scenix_material::{
    AlphaMode, FEATURE_ALBEDO_TEXTURE, FEATURE_CUSTOM_TEXTURES, FEATURE_DOUBLE_SIDED,
    FEATURE_NODE_GRAPH, Material, PipelineKey, ShaderKind,
};
use scenix_math::Vec3;

use crate::graph::{NodeDiagnostic, ShaderGraph};
use crate::node::{ShaderNodeKind, Stage};

/// Renderer-agnostic node-driven material: a [`ShaderGraph`] plus the live
/// CPU state feeding [`crate::node::UniformInput`] nodes.
///
/// The material implements the stable v1 [`Material`] trait, so node
/// materials integrate with pipeline keys, transparency sorting, and culling
/// exactly like built-in materials. The pipeline shader family is
/// [`ShaderKind::Custom`] keyed by the graph identity ([`ShaderGraph::shader_hash`]).
///
/// CPU state (base color, emissive, metallic, roughness) feeds the generated
/// shader live through the renderer's material uniform — editing state does
/// not recompile the shader. Editing the *graph* changes the shader identity
/// and requires re-registration.
///
/// # Example
/// ```
/// use scenix_core::Color;
/// use scenix_material::Material as _;
/// use scenix_nodes::{
///     LightingStyle, NodeMaterial, ShaderGraph, ShaderNodeKind, UniformInput,
/// };
///
/// let mut graph = ShaderGraph::new();
/// let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
/// let lit = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), base);
/// graph.add_fragment_output(lit, None);
///
/// let material = NodeMaterial::new("dirt", graph).with_base_color(Color::WHITE);
/// assert!(material.validate().is_empty());
/// assert_ne!(material.pipeline_key().shader, scenix_material::ShaderKind::Pbr,
///     "node materials are always Custom-shaded graphs");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeMaterial {
    name: String,
    graph: ShaderGraph,
    base_color: Color,
    emissive: Vec3,
    metallic: f32,
    roughness: f32,
    alpha_mode: AlphaMode,
    double_sided: bool,
    texture: Option<TextureId>,
}

impl NodeMaterial {
    /// Creates a node material from a graph with default live CPU state:
    /// white base color, zero emissive, metallic `0.0`, roughness `0.5`,
    /// opaque alpha, front faces, and no texture.
    #[must_use]
    pub fn new(name: impl Into<String>, graph: ShaderGraph) -> Self {
        Self {
            name: name.into(),
            graph,
            base_color: Color::WHITE,
            emissive: Vec3::default(),
            metallic: 0.0,
            roughness: 0.5,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
            texture: None,
        }
    }

    /// Sets the material name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the live base color feeding `UniformInput::BaseColor`.
    #[must_use]
    pub fn with_base_color(mut self, color: Color) -> Self {
        self.base_color = color;
        self
    }

    /// Sets the live emissive color feeding `UniformInput::Emissive`.
    #[must_use]
    pub fn with_emissive(mut self, emissive: Vec3) -> Self {
        self.emissive = emissive;
        self
    }

    /// Sets the live metallic factor feeding `UniformInput::Metallic`.
    #[must_use]
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic.clamp(0.0, 1.0);
        self
    }

    /// Sets the live roughness factor feeding `UniformInput::Roughness`.
    #[must_use]
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness.clamp(0.0, 1.0);
        self
    }

    /// Sets the alpha mode.
    #[must_use]
    pub fn with_alpha_mode(mut self, alpha_mode: AlphaMode) -> Self {
        self.alpha_mode = alpha_mode;
        self
    }

    /// Sets the alpha-test cutoff; `None` restores opaque, `Some` selects
    /// masked alpha with the given cutoff.
    #[must_use]
    pub fn with_alpha_cutoff(mut self, cutoff: Option<f32>) -> Self {
        self.alpha_mode = match cutoff {
            Some(cutoff) => AlphaMode::Mask(cutoff),
            None => AlphaMode::Opaque,
        };
        self
    }

    /// Renders double-sided when `true`.
    #[must_use]
    pub fn with_double_sided(mut self, double_sided: bool) -> Self {
        self.double_sided = double_sided;
        self
    }

    /// Binds the primary texture slot sampled by `TextureSample` nodes.
    #[must_use]
    pub fn with_texture(mut self, texture: Option<TextureId>) -> Self {
        self.texture = texture;
        self
    }

    /// Returns the material name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the material name in place.
    #[inline]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
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

    /// Returns the live base color.
    #[inline]
    pub fn base_color(&self) -> Color {
        self.base_color
    }

    /// Returns the live emissive color.
    #[inline]
    pub fn emissive(&self) -> Vec3 {
        self.emissive
    }

    /// Returns the live metallic factor.
    #[inline]
    pub fn metallic(&self) -> f32 {
        self.metallic
    }

    /// Returns the live roughness factor.
    #[inline]
    pub fn roughness(&self) -> f32 {
        self.roughness
    }

    /// Returns the alpha mode.
    #[inline]
    pub fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    /// Returns the alpha-test cutoff, when masked.
    #[inline]
    pub fn alpha_cutoff(&self) -> Option<f32> {
        match self.alpha_mode {
            AlphaMode::Opaque | AlphaMode::Blend => None,
            AlphaMode::Mask(cutoff) => Some(cutoff),
        }
    }

    /// Returns whether back-face culling is disabled.
    #[inline]
    pub fn double_sided(&self) -> bool {
        self.double_sided
    }

    /// Returns the bound primary texture slot, if any.
    #[inline]
    pub fn texture(&self) -> Option<TextureId> {
        self.texture
    }

    /// Returns whether the graph samples `TextureSample`.
    pub fn uses_texture(&self) -> bool {
        self.graph.ids().any(|id| {
            self.graph
                .get(id)
                .is_some_and(|node| matches!(node.kind, ShaderNodeKind::TextureSample))
        })
    }

    /// Returns the material feature flags reported through `pipeline_key`.
    pub fn feature_bits(&self) -> u64 {
        let mut bits = FEATURE_NODE_GRAPH;
        if self.double_sided {
            bits |= FEATURE_DOUBLE_SIDED;
        }
        if self.texture.is_some() {
            bits |= FEATURE_ALBEDO_TEXTURE | FEATURE_CUSTOM_TEXTURES;
        }
        bits
    }

    /// Returns the stable graph identity used by the pipeline key.
    #[inline]
    pub fn shader_hash(&self) -> u64 {
        self.graph.shader_hash()
    }

    /// Validates the live subgraph for the material fragment stage.
    pub fn validate(&self) -> Vec<NodeDiagnostic> {
        self.graph.validate(Stage::Fragment)
    }

    /// Replaces the graph; callers should [`NodeMaterial::validate`] after.
    #[inline]
    pub fn set_graph(&mut self, graph: ShaderGraph) {
        self.graph = graph;
    }
}

impl Material for NodeMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        PipelineKey::new(
            ShaderKind::Custom(self.graph.shader_hash()),
            self.alpha_mode.pipeline_alpha(),
            self.feature_bits(),
        )
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        self.alpha_mode.is_transparent()
    }

    #[inline]
    fn double_sided(&self) -> bool {
        self.double_sided
    }

    #[inline]
    fn alpha_cutoff(&self) -> Option<f32> {
        self.alpha_cutoff()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::UniformInput;

    fn shaded_graph() -> ShaderGraph {
        let mut graph = ShaderGraph::new();
        let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
        let lit = graph.chain(
            ShaderNodeKind::Lighting(crate::node::LightingStyle::Lambert),
            base,
        );
        graph.add_fragment_output(lit, None);
        graph
    }

    #[test]
    fn pipeline_key_reflects_graph_identity_and_state() {
        let graph_a = shaded_graph();
        let material_a = NodeMaterial::new("a", graph_a);
        let key_a = material_a.pipeline_key();
        assert!(key_a.shader != ShaderKind::Custom(0));
        assert!(!material_a.is_transparent());

        let mut graph_b = shaded_graph();
        graph_b.push(ShaderNodeKind::Uniform(UniformInput::Emissive));
        let material_b = NodeMaterial::new("b", graph_b);
        assert_ne!(
            material_a.pipeline_key().shader,
            material_b.pipeline_key().shader
        );
    }

    #[test]
    fn feature_bits_track_textures_and_double_sided() {
        let material = NodeMaterial::new("t", shaded_graph());
        assert_eq!(material.feature_bits(), FEATURE_NODE_GRAPH);

        let textured = material
            .clone()
            .with_double_sided(true)
            .with_texture(Some(TextureId::new(7)));
        assert!(textured.feature_bits() & FEATURE_ALBEDO_TEXTURE != 0);
        assert!(textured.feature_bits() & FEATURE_CUSTOM_TEXTURES != 0);
        assert!(textured.feature_bits() & FEATURE_DOUBLE_SIDED != 0);
    }

    #[test]
    fn valid_graph_passes_validation() {
        let material = NodeMaterial::new("v", shaded_graph());
        assert!(material.validate().is_empty());
    }
}
