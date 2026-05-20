use scenix::{
    AlphaMode, FEATURE_ALBEDO_TEXTURE, Material, PbrMaterial, ShaderKind, ShaderMaterial, TextureId,
};

fn main() {
    let opaque = PbrMaterial::new();
    let mut textured = PbrMaterial::new();
    textured.albedo_texture = Some(TextureId::new(7));

    let masked = PbrMaterial::new().alpha_mode(AlphaMode::Mask(0.5));
    let custom = ShaderMaterial::new("@vertex fn vs() {}", "@fragment fn fs() {}");

    assert_ne!(opaque.pipeline_key(), textured.pipeline_key());
    assert!(textured.pipeline_key().has_feature(FEATURE_ALBEDO_TEXTURE));
    assert_eq!(masked.alpha_cutoff(), Some(0.5));
    assert!(matches!(
        custom.pipeline_key().shader,
        ShaderKind::Custom(_)
    ));
}
