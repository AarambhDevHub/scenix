#![cfg(feature = "serde")]

use scenix_core::Color;
use scenix_light::{
    AmbientLight, AreaLight, DirectionalLight, HemisphereLight, LightProbe, PointLight,
    ShadowConfig, SpotLight,
};
use scenix_math::Vec3;

#[test]
fn light_types_round_trip_with_serde() {
    let shadow = ShadowConfig::new(2048, 0.1, 250.0)
        .pcf_radius(2)
        .cascades(3);
    let ambient = AmbientLight::new(Color::WHITE, 0.2);
    let directional =
        DirectionalLight::new(Vec3::new(1.0, -1.0, 0.0), Color::WHITE, 4.0).shadow(shadow);
    let point = PointLight::new(Color::RED, 3.0, 20.0).shadow(shadow);
    let spot = SpotLight::new(Color::GREEN, 5.0, 50.0, 0.7).penumbra(0.5);
    let hemisphere = HemisphereLight::new(Color::BLUE, Color::GREEN, 0.4);
    let area = AreaLight::new(2.0, 3.0, Color::WHITE, 6.0);
    let probe = LightProbe::from_coefficients([Vec3::ONE; 9], 0.75);

    assert_eq!(
        serde_json::from_str::<AmbientLight>(&serde_json::to_string(&ambient).unwrap()).unwrap(),
        ambient
    );
    assert_eq!(
        serde_json::from_str::<DirectionalLight>(&serde_json::to_string(&directional).unwrap())
            .unwrap(),
        directional
    );
    assert_eq!(
        serde_json::from_str::<PointLight>(&serde_json::to_string(&point).unwrap()).unwrap(),
        point
    );
    assert_eq!(
        serde_json::from_str::<SpotLight>(&serde_json::to_string(&spot).unwrap()).unwrap(),
        spot
    );
    assert_eq!(
        serde_json::from_str::<HemisphereLight>(&serde_json::to_string(&hemisphere).unwrap())
            .unwrap(),
        hemisphere
    );
    assert_eq!(
        serde_json::from_str::<AreaLight>(&serde_json::to_string(&area).unwrap()).unwrap(),
        area
    );
    assert_eq!(
        serde_json::from_str::<LightProbe>(&serde_json::to_string(&probe).unwrap()).unwrap(),
        probe
    );
}
