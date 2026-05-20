use scenix_core::{Color, LightId, ValidationError};
use scenix_light::{
    AmbientLight, AreaLight, DirectionalLight, HemisphereLight, PointLight, ShadowConfig, SpotLight,
};
use scenix_math::Vec3;
use scenix_scene::{NodeKind, SceneNode};

fn close(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-4, "{a} != {b}");
}

#[test]
fn light_constructors_normalize_and_clamp_values() {
    let directional = DirectionalLight::new(Vec3::new(0.0, -2.0, 0.0), Color::WHITE, 4.0);
    close(directional.direction.length(), 1.0);

    let fallback = DirectionalLight::new(Vec3::ZERO, Color::WHITE, 1.0);
    assert_eq!(fallback.direction, Vec3::NEG_Z);

    let point = PointLight::new(Color::WHITE, 2.0, -1.0).decay(-2.0);
    assert_eq!(point.range, 0.0);
    assert_eq!(point.decay, 2.0);

    let spot = SpotLight::new(Color::WHITE, 2.0, -1.0, -0.5).penumbra(2.0);
    assert_eq!(spot.range, 0.0);
    assert_eq!(spot.angle, 0.0);
    assert_eq!(spot.penumbra, 1.0);

    let area = AreaLight::new(0.0, -1.0, Color::WHITE, 3.0);
    assert_eq!(area.width, 1.0);
    assert_eq!(area.height, 1.0);
}

#[test]
fn shadow_config_validates_ranges() {
    assert!(ShadowConfig::new(1024, 0.1, 100.0).validate().is_ok());
    assert_eq!(
        ShadowConfig::new(1000, 0.1, 100.0).validate(),
        Err(ValidationError::OutOfRange)
    );
    assert_eq!(
        ShadowConfig::new(1024, 1.0, 0.5).validate(),
        Err(ValidationError::OutOfRange)
    );
    assert_eq!(
        ShadowConfig::new(1024, 0.1, 100.0).cascades(5).validate(),
        Err(ValidationError::OutOfRange)
    );
}

#[test]
fn scene_nodes_can_attach_lights_by_id() {
    let light_id = LightId::new(7);
    let node = SceneNode::light("key", light_id);

    assert_eq!(node.kind, NodeKind::Light { light_id });

    let ambient = AmbientLight::new(Color::WHITE, 0.25);
    let hemisphere = HemisphereLight::new(Color::BLUE, Color::GREEN, 0.5);

    assert_eq!(ambient.intensity, 0.25);
    assert_eq!(hemisphere.intensity, 0.5);
}
