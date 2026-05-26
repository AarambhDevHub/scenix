use std::hint::black_box;
use std::time::Instant;

use scenix_core::Color;
use scenix_helpers::{AxesHelper, BoundingBoxHelper, GridHelper, LineGeometry};
use scenix_math::{Aabb, Vec3};

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
}
