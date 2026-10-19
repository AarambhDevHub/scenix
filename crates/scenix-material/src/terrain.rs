//! Terrain splat material descriptor (scenix v1.8.0 "Terrain, Sky, And
//! Water").
//!
//! A [`TerrainMaterial`] describes a four-layer texture surface driven by a
//! splat map: each layer carries a fallback albedo, roughness, and tiling
//! factor, and the RGBA splat texture (or per-vertex color weights) decides
//! the per-pixel blend. The renderer `environment` feature binds the splat
//! texture and runs the `terrain_splat` WGSL pipeline; the CPU geometry
//! from `scenix-terrain` rides the same vertex-color channel when no splat
//! texture is bound.

use scenix_core::{Color, TextureId};

use crate::traits::double_sided_bit;
use crate::{
    FEATURE_ALBEDO_TEXTURE, FEATURE_SPLAT_MAP, FEATURE_VERTEX_COLORS, Material, PipelineAlphaMode,
    PipelineKey, ShaderKind,
};

/// One terrain surface layer descriptor.
///
/// `SplatLayerDesc` is the material-side twin of
/// `scenix_terrain::SplatLayer` (scenix-material stays dependency-light, so
/// the layer data is duplicated just like the v1.7 flipbook parameters
/// were).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SplatLayerDesc {
    /// Layer albedo, also the fallback when no textures are bound.
    pub albedo: Color,
    /// Surface roughness, clamped into `0.05..=1.0` on construction.
    pub roughness: f32,
    /// Texture repeats across the whole terrain map (`0` disables repeats).
    pub tiling: f32,
}

impl SplatLayerDesc {
    /// Creates a clamped layer descriptor.
    #[inline]
    #[must_use]
    pub fn new(albedo: Color, roughness: f32, tiling: f32) -> Self {
        Self {
            albedo,
            roughness: roughness.clamp(0.05, 1.0),
            tiling: tiling.max(0.0),
        }
    }

    /// A mid-grass green fallback layer.
    #[inline]
    #[must_use]
    pub const fn default_grass() -> Self {
        Self {
            albedo: Color::rgb(0.28, 0.42, 0.16),
            roughness: 0.92,
            tiling: 24.0,
        }
    }

    /// A gray rock fallback layer.
    #[inline]
    #[must_use]
    pub const fn default_rock() -> Self {
        Self {
            albedo: Color::rgb(0.46, 0.45, 0.43),
            roughness: 0.78,
            tiling: 18.0,
        }
    }

    /// A brown soil fallback layer.
    #[inline]
    #[must_use]
    pub const fn default_dirt() -> Self {
        Self {
            albedo: Color::rgb(0.38, 0.28, 0.18),
            roughness: 0.85,
            tiling: 20.0,
        }
    }

    /// A pale sand fallback layer.
    #[inline]
    #[must_use]
    pub const fn default_sand() -> Self {
        Self {
            albedo: Color::rgb(0.80, 0.72, 0.50),
            roughness: 0.95,
            tiling: 16.0,
        }
    }

    /// The four standard layers in splat channel order `(w0..w3)`.
    #[inline]
    #[must_use]
    pub const fn defaults() -> [Self; 4] {
        [
            Self::default_grass(),
            Self::default_rock(),
            Self::default_dirt(),
            Self::default_sand(),
        ]
    }
}

/// Four-layer splat-map terrain material.
///
/// The albedo texture feature fires when a splat texture is bound; the
/// vertex-color feature mirrors the `TerrainGrid::chunk_geometry_with_splat`
/// CPU path where weights ride the vertex color channel rather than a
/// texture.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TerrainMaterial {
    /// Blend layers in splat channel order.
    pub layers: [SplatLayerDesc; 4],
    /// Registered RGBA splat-weight texture (weights normalized in `1.0`
    /// sums; channels map `r = w0`, `g = w1`, `b = w2`, `a = w3`).
    pub splat_texture: Option<TextureId>,
    /// Optional tiled detail texture multiplied over the blended albedo.
    pub detail_texture: Option<TextureId>,
    /// Whether to cull back faces (terrain is usually double-sided).
    pub double_sided: bool,
}

impl TerrainMaterial {
    /// A grass/rock/dirt/sand material with the default layer palette.
    #[inline]
    #[must_use]
    pub const fn default_palette() -> Self {
        Self {
            layers: SplatLayerDesc::defaults(),
            splat_texture: None,
            detail_texture: None,
            double_sided: true,
        }
    }

    /// Returns the material with fully custom layers.
    #[inline]
    #[must_use]
    pub const fn with_layers(mut self, layers: [SplatLayerDesc; 4]) -> Self {
        self.layers = layers;
        self
    }

    /// Returns the material with a splat texture bound.
    #[inline]
    #[must_use]
    pub const fn with_splat_texture(mut self, texture_id: TextureId) -> Self {
        self.splat_texture = Some(texture_id);
        self
    }

    /// Returns the material with a tiled detail texture bound.
    #[inline]
    #[must_use]
    pub const fn with_detail_texture(mut self, texture_id: TextureId) -> Self {
        self.detail_texture = Some(texture_id);
        self
    }

    /// Returns the material with explicit back-face culling.
    #[inline]
    #[must_use]
    pub const fn with_double_sided(mut self, double_sided: bool) -> Self {
        self.double_sided = double_sided;
        self
    }

    /// The dominant preview color (layer 0), used by editor previews.
    #[inline]
    #[must_use]
    pub const fn preview_color(&self) -> Color {
        self.layers[0].albedo
    }
}

impl Material for TerrainMaterial {
    fn pipeline_key(&self) -> PipelineKey {
        let mut bits = double_sided_bit(self.double_sided);
        if self.splat_texture.is_some() {
            bits |= FEATURE_SPLAT_MAP;
        }
        if self.detail_texture.is_some() {
            bits |= FEATURE_ALBEDO_TEXTURE;
        }
        // Vertex-color splat weights are the documented CPU fallback even
        // without a splat texture.
        bits |= FEATURE_VERTEX_COLORS;
        PipelineKey::new(ShaderKind::Terrain, PipelineAlphaMode::Opaque, bits)
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        false
    }

    #[inline]
    fn double_sided(&self) -> bool {
        self.double_sided
    }

    #[inline]
    fn alpha_cutoff(&self) -> Option<f32> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{SplatLayerDesc, TerrainMaterial};
    use crate::{
        FEATURE_ALBEDO_TEXTURE, FEATURE_SPLAT_MAP, FEATURE_VERTEX_COLORS, Material,
        PipelineAlphaMode, ShaderKind,
    };
    use scenix_core::{Color, TextureId};

    #[test]
    fn terrain_pipeline_routes_splat_features() {
        let bare = TerrainMaterial::default_palette();
        assert!(bare.pipeline_key().has_feature(FEATURE_VERTEX_COLORS));
        assert!(!bare.pipeline_key().has_feature(FEATURE_SPLAT_MAP));
        assert_eq!(bare.pipeline_key().shader, ShaderKind::Terrain);
        assert_eq!(bare.pipeline_key().alpha, PipelineAlphaMode::Opaque);
        assert!(!bare.is_transparent());
        assert!(bare.double_sided());
        assert_eq!(bare.alpha_cutoff(), None);

        let textured = bare
            .with_splat_texture(TextureId::new(4))
            .with_detail_texture(TextureId::new(9));
        assert!(textured.pipeline_key().has_feature(FEATURE_SPLAT_MAP));
        assert!(textured.pipeline_key().has_feature(FEATURE_ALBEDO_TEXTURE));
    }

    #[test]
    fn layers_clamp_and_order() {
        let tuned = SplatLayerDesc::new(Color::WHITE, 9.0, -2.0);
        assert_eq!((tuned.roughness, tuned.tiling), (1.0, 0.0));
        let defaults = SplatLayerDesc::defaults();
        assert_eq!(defaults.len(), 4);
        assert_ne!(defaults[0].albedo, defaults[1].albedo);
    }
}
