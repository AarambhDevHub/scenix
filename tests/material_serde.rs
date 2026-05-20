#![cfg(feature = "serde")]

use scenix_core::{Color, TextureId};
use scenix_material::{
    AlphaMode, PbrMaterial, PhysicalMaterial, ShaderMaterial, ToonMaterial, UnlitMaterial,
};

#[test]
fn material_types_round_trip_with_serde() {
    let mut pbr = PbrMaterial::new()
        .albedo(Color::from_hex(0xAA_55_11))
        .alpha_mode(AlphaMode::Mask(0.5));
    pbr.albedo_texture = Some(TextureId::new(3));

    let physical = PhysicalMaterial::new()
        .base(pbr.clone())
        .clearcoat(1.0, 0.25)
        .transmission(0.4, 0.1);
    let unlit = UnlitMaterial::new().color(Color::BLUE);
    let toon = ToonMaterial::new().steps(5).outline(0.03, Color::BLACK);
    let mut shader = ShaderMaterial::new("@vertex fn vs() {}", "@fragment fn fs() {}");
    shader.uniforms = vec![1, 2, 3, 4];
    shader.textures.push(TextureId::new(8));

    assert_eq!(
        serde_json::from_str::<PbrMaterial>(&serde_json::to_string(&pbr).unwrap()).unwrap(),
        pbr
    );
    assert_eq!(
        serde_json::from_str::<PhysicalMaterial>(&serde_json::to_string(&physical).unwrap())
            .unwrap(),
        physical
    );
    assert_eq!(
        serde_json::from_str::<UnlitMaterial>(&serde_json::to_string(&unlit).unwrap()).unwrap(),
        unlit
    );
    assert_eq!(
        serde_json::from_str::<ToonMaterial>(&serde_json::to_string(&toon).unwrap()).unwrap(),
        toon
    );
    assert_eq!(
        serde_json::from_str::<ShaderMaterial>(&serde_json::to_string(&shader).unwrap()).unwrap(),
        shader
    );
}
