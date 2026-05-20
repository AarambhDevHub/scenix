use scenix_core::{Color, TextureId};
use scenix_material::{
    AlphaMode, FEATURE_ALBEDO_TEXTURE, FEATURE_CLEARCOAT, FEATURE_GRADIENT_TEXTURE,
    FEATURE_OUTLINE, FEATURE_TRANSMISSION, Material, PbrMaterial, PhysicalMaterial,
    PipelineAlphaMode, ShaderKind, ShaderMaterial, ToonMaterial,
};

#[test]
fn pipeline_keys_distinguish_material_configurations() {
    let opaque = PbrMaterial::new();
    let mut textured = PbrMaterial::new();
    textured.albedo_texture = Some(TextureId::new(1));

    assert_ne!(opaque.pipeline_key(), textured.pipeline_key());
    assert!(textured.pipeline_key().has_feature(FEATURE_ALBEDO_TEXTURE));

    let toon = ToonMaterial::new().outline(0.02, Color::BLACK);
    assert!(toon.pipeline_key().has_feature(FEATURE_OUTLINE));
}

#[test]
fn alpha_modes_report_transparency_and_cutoffs() {
    let masked = PbrMaterial::new().alpha_mode(AlphaMode::Mask(0.42));
    let blended = PbrMaterial::new().alpha_mode(AlphaMode::Blend);

    assert_eq!(masked.pipeline_key().alpha, PipelineAlphaMode::Mask);
    assert_eq!(masked.alpha_cutoff(), Some(0.42));
    assert!(!masked.is_transparent());

    assert_eq!(blended.pipeline_key().alpha, PipelineAlphaMode::Blend);
    assert!(blended.is_transparent());
    assert_eq!(blended.alpha_cutoff(), None);
}

#[test]
fn physical_material_features_are_compact_flags() {
    let material = PhysicalMaterial::new()
        .clearcoat(1.0, 0.2)
        .transmission(0.6, 0.1);
    let key = material.pipeline_key();

    assert_eq!(key.shader, ShaderKind::Physical);
    assert_eq!(key.alpha, PipelineAlphaMode::Blend);
    assert!(key.has_feature(FEATURE_CLEARCOAT));
    assert!(key.has_feature(FEATURE_TRANSMISSION));
    assert!(material.is_transparent());
}

#[test]
fn shader_material_uses_stable_source_ids() {
    let a = ShaderMaterial::new("@vertex fn vs() {}", "@fragment fn fs() {}");
    let b = ShaderMaterial::new("@vertex fn vs() {}", "@fragment fn fs() {}");
    let c = ShaderMaterial::new("@vertex fn other() {}", "@fragment fn fs() {}");

    assert_eq!(a.shader_id(), b.shader_id());
    assert_ne!(a.shader_id(), c.shader_id());
    assert_eq!(a.pipeline_key(), b.pipeline_key());
    assert_ne!(a.pipeline_key(), c.pipeline_key());
    assert!(matches!(a.pipeline_key().shader, ShaderKind::Custom(_)));
}

#[test]
fn toon_texture_features_are_distinct() {
    let mut toon = ToonMaterial::new();
    toon.gradient_map = Some(TextureId::new(9));

    assert!(toon.pipeline_key().has_feature(FEATURE_GRADIENT_TEXTURE));
}
