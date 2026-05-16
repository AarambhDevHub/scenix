use std::hint::black_box;
use std::time::Instant;

use scenix_math::{Aabb, Mat4, Quat, Ray3, Transform, Vec3};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn main() {
    let a = Mat4::from_trs(
        Vec3::new(1.0, 2.0, 3.0),
        Quat::from_axis_angle(Vec3::Y, 0.7),
        Vec3::new(2.0, 2.0, 2.0),
    );
    let b = Mat4::from_trs(
        Vec3::new(4.0, 5.0, 6.0),
        Quat::from_axis_angle(Vec3::X, 0.4),
        Vec3::new(0.5, 0.5, 0.5),
    );
    bench("mat4_mul", 100_000, || {
        black_box(black_box(a) * black_box(b));
    });

    let q0 = Quat::IDENTITY;
    let q1 = Quat::from_axis_angle(Vec3::Y, core::f32::consts::PI);
    bench("quat_slerp", 100_000, || {
        black_box(black_box(q0).slerp(black_box(q1), 0.42));
    });

    let ray = Ray3::new(Vec3::new(0.0, 0.0, 5.0), Vec3::NEG_Z);
    let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    bench("ray_aabb", 100_000, || {
        black_box(black_box(ray).intersect_aabb(black_box(aabb)));
    });

    let t0 = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let t1 = Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, 0.3));
    bench("transform_compose", 100_000, || {
        black_box(black_box(t0).mul_transform(black_box(t1)));
    });
}
