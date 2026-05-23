#![cfg(feature = "serde")]

use scenix_camera::{
    CubeCamera, FlyController, Frustum, OrbitController, OrthographicCamera, PerspectiveCamera,
};
use scenix_math::{Mat4, Vec3};

#[test]
fn camera_types_round_trip_with_serde() {
    let perspective = PerspectiveCamera::new(60.0, 1.5, 0.1, 100.0)
        .position(Vec3::new(1.0, 2.0, 3.0))
        .target(Vec3::ZERO);
    let orthographic = OrthographicCamera::new(-2.0, 2.0, -1.0, 1.0, 0.1, 20.0);
    let cube = CubeCamera::new(Vec3::new(1.0, 2.0, 3.0), 0.1, 50.0);
    let orbit = OrbitController::new(Vec3::ZERO, 5.0);
    let fly = FlyController::new(Vec3::new(0.0, 1.0, 2.0));
    let frustum = Frustum::from_view_projection(Mat4::IDENTITY);

    assert_eq!(
        serde_json::from_str::<PerspectiveCamera>(&serde_json::to_string(&perspective).unwrap())
            .unwrap(),
        perspective
    );
    assert_eq!(
        serde_json::from_str::<OrthographicCamera>(&serde_json::to_string(&orthographic).unwrap())
            .unwrap(),
        orthographic
    );
    assert_eq!(
        serde_json::from_str::<CubeCamera>(&serde_json::to_string(&cube).unwrap()).unwrap(),
        cube
    );
    assert_eq!(
        serde_json::from_str::<OrbitController>(&serde_json::to_string(&orbit).unwrap()).unwrap(),
        orbit
    );
    assert_eq!(
        serde_json::from_str::<FlyController>(&serde_json::to_string(&fly).unwrap()).unwrap(),
        fly
    );
    assert_eq!(
        serde_json::from_str::<Frustum>(&serde_json::to_string(&frustum).unwrap()).unwrap(),
        frustum
    );
}
