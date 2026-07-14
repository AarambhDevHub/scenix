use std::hint::black_box;
use std::time::Instant;

use scenix_core::Color;
use scenix_helpers::{
    AxesHelper, BoundingBoxHelper, GizmoGeometry, GridHelper, LineGeometry, TransformGizmoHelper,
};
use scenix_math::{Aabb, Ray3, Vec3};
use scenix_scene::TransformMode;

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn main() {
    bench("grid_helper_256", 10_000, || {
        black_box(GridHelper::new(100.0, 256).to_geometry());
    });

    bench("helper_merge", 10_000, || {
        let mut lines = LineGeometry::new();
        lines.merge(&GridHelper::new(10.0, 10).to_geometry());
        lines.merge(&AxesHelper::new(1.0).to_geometry());
        lines.merge(
            &BoundingBoxHelper::new(Aabb::new(-Vec3::ONE, Vec3::ONE), Color::WHITE).to_geometry(),
        );
        black_box(lines);
    });

    let helper = TransformGizmoHelper::new(Vec3::ZERO, TransformMode::Translate);
    let mut geometry = GizmoGeometry::default();
    let ray = Ray3::new(Vec3::new(1.0, 0.04, 4.0), Vec3::NEG_Z);
    bench("transform_gizmo_reused_geometry_and_hit", 100_000, || {
        helper.write_geometry(black_box(&mut geometry));
        black_box(geometry.hit_test(black_box(ray)));
    });
}
