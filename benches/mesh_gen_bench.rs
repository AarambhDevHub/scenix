use std::hint::black_box;
use std::time::Instant;

use scenix_mesh::{Geometry, box_geometry, icosphere_geometry, sphere_geometry, torus_geometry};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn main() {
    bench("box_geometry", 10_000, || {
        black_box(box_geometry(1.0, 1.0, 1.0, 4, 4, 4));
    });

    bench("sphere_geometry", 2_000, || {
        black_box(sphere_geometry(1.0, 32, 16));
    });

    bench("torus_geometry", 2_000, || {
        black_box(torus_geometry(1.0, 0.25, 16, 32));
    });

    bench("icosphere_geometry", 500, || {
        black_box(icosphere_geometry(1.0, 3));
    });

    bench("compute_tangents", 2_000, || {
        let mut geometry = sphere_geometry(1.0, 32, 16);
        geometry.compute_tangents();
        black_box(geometry);
    });

    let a = sphere_geometry(1.0, 16, 8);
    let b = torus_geometry(1.0, 0.2, 12, 24);
    bench("geometry_merge", 10_000, || {
        let mut merged = Geometry::new();
        merged.merge(black_box(&a));
        merged.merge(black_box(&b));
        black_box(merged);
    });
}
