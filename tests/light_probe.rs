use scenix_core::ValidationError;
use scenix_light::LightProbe;
use scenix_math::Vec3;

#[test]
fn cube_face_projection_produces_non_zero_coefficients() {
    let px = [Vec3::new(1.0, 0.0, 0.0); 4];
    let nx = [Vec3::new(0.0, 1.0, 0.0); 4];
    let py = [Vec3::new(0.0, 0.0, 1.0); 4];
    let ny = [Vec3::new(0.1, 0.1, 0.1); 4];
    let pz = [Vec3::new(1.0, 1.0, 1.0); 4];
    let nz = [Vec3::new(0.2, 0.4, 0.8); 4];

    let probe = LightProbe::from_cube_faces([&px, &nx, &py, &ny, &pz, &nz], 2, 1.0).unwrap();

    assert!(probe.sh_coefficients.iter().any(|coefficient| {
        coefficient.x != 0.0 || coefficient.y != 0.0 || coefficient.z != 0.0
    }));
}

#[test]
fn equirectangular_projection_produces_non_zero_coefficients() {
    let samples = vec![Vec3::new(0.5, 0.75, 1.0); 8];
    let probe = LightProbe::from_equirectangular_samples(&samples, 4, 2, 0.8).unwrap();

    assert_eq!(probe.intensity, 0.8);
    assert!(probe.sh_coefficients[0].x > 0.0);
}

#[test]
fn projection_rejects_malformed_inputs() {
    let face = [Vec3::ONE; 3];
    assert_eq!(
        LightProbe::from_cube_faces([&face, &face, &face, &face, &face, &face], 2, 1.0),
        Err(ValidationError::InvalidState)
    );

    assert_eq!(
        LightProbe::from_equirectangular_samples(&[Vec3::ONE], 2, 2, 1.0),
        Err(ValidationError::InvalidState)
    );
}
