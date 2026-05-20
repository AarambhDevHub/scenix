use scenix::{LightProbe, Vec3};

fn main() {
    let px = [Vec3::new(1.0, 0.2, 0.2); 4];
    let nx = [Vec3::new(0.2, 1.0, 0.2); 4];
    let py = [Vec3::new(0.2, 0.2, 1.0); 4];
    let ny = [Vec3::new(0.1, 0.1, 0.1); 4];
    let pz = [Vec3::new(1.0, 1.0, 1.0); 4];
    let nz = [Vec3::new(0.4, 0.4, 0.8); 4];

    let probe = LightProbe::from_cube_faces([&px, &nx, &py, &ny, &pz, &nz], 2, 1.0).unwrap();

    assert!(probe.sh_coefficients.iter().any(|coefficient| {
        coefficient.x != 0.0 || coefficient.y != 0.0 || coefficient.z != 0.0
    }));
}
