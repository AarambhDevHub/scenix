use scenix_camera::{OrthographicCamera, PerspectiveCamera};
use scenix_core::Color;
use scenix_helpers::{
    ArrowHelper, AxesHelper, BoundingBoxHelper, CameraHelper, DirectionalLightHelper, GridHelper,
    LineGeometry, PointLightHelper, SkeletonHelper, SpotLightHelper,
};
use scenix_light::{DirectionalLight, PointLight, SpotLight};
use scenix_math::{Aabb, Vec3};

#[test]
fn line_geometry_validates_pairs_indices_and_colors() {
    let mut geometry = LineGeometry::new();
    geometry.push_segment(Vec3::ZERO, Vec3::X, Color::WHITE);
    geometry.validate().unwrap();
    assert_eq!(geometry.segment_count(), 1);

    geometry.positions.push(Vec3::Y);
    assert!(geometry.validate().is_err());
}

#[test]
fn line_geometry_merges_with_offset_indices() {
    let mut a = LineGeometry::new();
    a.push_segment(Vec3::ZERO, Vec3::X, Color::RED);
    a.indices = vec![0, 1];
    let mut b = LineGeometry::new();
    b.push_segment(Vec3::Y, Vec3::Z, Color::BLUE);
    b.indices = vec![0, 1];
    a.merge(&b);
    a.validate().unwrap();
    assert_eq!(a.indices, vec![0, 1, 2, 3]);
}

#[test]
fn grid_axes_box_and_arrow_helpers_generate_expected_segments() {
    let grid = GridHelper::new(10.0, 10).to_geometry();
    grid.validate().unwrap();
    assert_eq!(grid.segment_count(), 22);

    let axes = AxesHelper::new(2.0).to_geometry();
    axes.validate().unwrap();
    assert_eq!(axes.segment_count(), 3);
    assert_eq!(axes.colors[0], Color::RED);

    let bounds =
        BoundingBoxHelper::new(Aabb::new(-Vec3::ONE, Vec3::ONE), Color::WHITE).to_geometry();
    bounds.validate().unwrap();
    assert_eq!(bounds.segment_count(), 12);

    let arrow = ArrowHelper::new(Vec3::ZERO, Vec3::Y, 2.0, Color::GREEN).to_geometry();
    arrow.validate().unwrap();
    assert_eq!(arrow.segment_count(), 5);
}

#[test]
fn light_helpers_generate_deterministic_wireframes() {
    let point = PointLightHelper {
        segments: 8,
        ..PointLightHelper::new(
            PointLight::new(Color::WHITE, 1.0, 2.0),
            Vec3::ZERO,
            Color::WHITE,
        )
    }
    .to_geometry();
    point.validate().unwrap();
    assert_eq!(point.segment_count(), 24);

    let spot = SpotLightHelper {
        segments: 8,
        ..SpotLightHelper::new(
            SpotLight::new(Color::WHITE, 1.0, 3.0, core::f32::consts::FRAC_PI_4),
            Vec3::ZERO,
            Vec3::NEG_Z,
            Color::WHITE,
        )
    }
    .to_geometry();
    spot.validate().unwrap();
    assert_eq!(spot.segment_count(), 12);

    let directional = DirectionalLightHelper::new(
        DirectionalLight::new(Vec3::NEG_Z, Color::WHITE, 1.0),
        Vec3::ZERO,
        1.0,
        Color::WHITE,
    )
    .to_geometry();
    directional.validate().unwrap();
    assert_eq!(directional.segment_count(), 5);
}

#[test]
fn camera_helper_generates_frustum_edges() {
    let perspective = PerspectiveCamera::new(60.0, 1.0, 0.1, 10.0);
    let helper = CameraHelper::from_perspective(&perspective, Color::WHITE).to_geometry();
    helper.validate().unwrap();
    assert_eq!(helper.segment_count(), 12);

    let orthographic = OrthographicCamera::new(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0);
    let helper = CameraHelper::from_orthographic(&orthographic, Color::WHITE).to_geometry();
    helper.validate().unwrap();
    assert_eq!(helper.segment_count(), 12);
}

#[test]
fn skeleton_helper_validates_and_connects_parent_joints() {
    let helper = SkeletonHelper::new(
        vec![Vec3::ZERO, Vec3::Y, Vec3::new(0.0, 2.0, 0.0)],
        vec![None, Some(0), Some(1)],
        Color::WHITE,
    );
    helper.validate().unwrap();
    let geometry = helper.to_geometry();
    geometry.validate().unwrap();
    assert_eq!(geometry.segment_count(), 2);
}

#[cfg(feature = "serde")]
#[test]
fn helper_types_round_trip_with_serde() {
    let helper = GridHelper::new(4.0, 4).colors(Color::RED, Color::BLUE);
    let json = serde_json::to_string(&helper).unwrap();
    let out: GridHelper = serde_json::from_str(&json).unwrap();
    assert_eq!(out, helper);
}
