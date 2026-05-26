use scenix::{
    Aabb, ArrowHelper, AxesHelper, BoundingBoxHelper, CameraHelper, Color, DirectionalLight,
    DirectionalLightHelper, GridHelper, LineGeometry, PerspectiveCamera, PointLight,
    PointLightHelper, SkeletonHelper, SpotLight, SpotLightHelper, Vec3,
};

fn main() {
    let mut lines = LineGeometry::new();
    lines.merge(&GridHelper::new(10.0, 10).to_geometry());
    lines.merge(&AxesHelper::new(2.0).to_geometry());
    lines.merge(
        &BoundingBoxHelper::new(Aabb::new(-Vec3::ONE, Vec3::ONE), Color::WHITE).to_geometry(),
    );
    lines.merge(&ArrowHelper::new(Vec3::ZERO, Vec3::Y, 2.5, Color::GREEN).to_geometry());

    let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 20.0)
        .position(Vec3::new(0.0, 1.0, 5.0))
        .target(Vec3::ZERO);
    lines.merge(&CameraHelper::from_perspective(&camera, Color::from_hex(0x66CCFF)).to_geometry());

    lines.merge(
        &PointLightHelper::new(
            PointLight::new(Color::WHITE, 1.0, 1.5),
            Vec3::new(-2.0, 1.0, 0.0),
            Color::from_hex(0xFFDD66),
        )
        .to_geometry(),
    );
    lines.merge(
        &SpotLightHelper::new(
            SpotLight::new(Color::WHITE, 1.0, 2.0, core::f32::consts::FRAC_PI_4),
            Vec3::new(2.0, 1.0, 0.0),
            Vec3::NEG_Z,
            Color::from_hex(0xFF9966),
        )
        .to_geometry(),
    );
    lines.merge(
        &DirectionalLightHelper::new(
            DirectionalLight::new(Vec3::NEG_Z, Color::WHITE, 1.0),
            Vec3::new(0.0, 2.0, 0.0),
            1.5,
            Color::from_hex(0xCCCCFF),
        )
        .to_geometry(),
    );
    lines.merge(
        &SkeletonHelper::new(
            vec![Vec3::ZERO, Vec3::Y, Vec3::new(0.5, 1.75, 0.0)],
            vec![None, Some(0), Some(1)],
            Color::from_hex(0xFF66CC),
        )
        .to_geometry(),
    );

    lines.validate().unwrap();
    println!(
        "generated {} helper vertices across {} line segments",
        lines.vertex_count(),
        lines.segment_count()
    );
}
