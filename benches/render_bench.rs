use std::hint::black_box;
use std::time::Instant;

use scenix_camera::PerspectiveCamera;
use scenix_core::{MaterialId, MeshId};
use scenix_material::PbrMaterial;
use scenix_math::{Transform, Vec3};
use scenix_mesh::box_geometry;
use scenix_renderer::{Renderer, RendererConfig};
use scenix_scene::{SceneGraph, SceneNode};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn scene_with_triangle_budget(triangles: usize) -> SceneGraph {
    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(1);
    let cube_triangles = 12;
    let nodes = triangles.div_ceil(cube_triangles).max(1);
    let mut scene = SceneGraph::new();

    for index in 0..nodes {
        let x = (index % 64) as f32 * 0.18 - 5.5;
        let y = ((index / 64) % 64) as f32 * 0.18 - 5.5;
        let z = (index / 4096) as f32 * -0.2;
        scene.add(
            SceneNode::mesh(format!("cube-{index}"), mesh_id, material_id)
                .transform(Transform::from_translation(Vec3::new(x, y, z))),
        );
    }
    scene.update_world_transforms();
    scene
}

fn main() {
    if std::env::var("SCENIX_RUN_GPU_BENCHES").as_deref() != Ok("1") {
        println!("set SCENIX_RUN_GPU_BENCHES=1 to run renderer GPU benches");
        return;
    }

    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(128, 128))
            .await
            .expect("headless renderer");
        renderer
            .register_mesh(MeshId::new(1), &box_geometry(0.1, 0.1, 0.1, 1, 1, 1))
            .expect("mesh upload");
        renderer
            .register_pbr_material(MaterialId::new(1), &PbrMaterial::new())
            .expect("material upload");

        let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 10.0))
            .target(Vec3::ZERO);
        let scene_1k = scene_with_triangle_budget(1_000);
        let scene_10k = scene_with_triangle_budget(10_000);
        let scene_100k = scene_with_triangle_budget(100_000);

        bench("renderer_headless_1k_triangles", 200, || {
            black_box(
                renderer
                    .render(black_box(&scene_1k), black_box(&camera))
                    .unwrap(),
            );
        });
        bench("renderer_headless_10k_triangles", 100, || {
            black_box(
                renderer
                    .render(black_box(&scene_10k), black_box(&camera))
                    .unwrap(),
            );
        });
        bench("renderer_headless_100k_triangles", 20, || {
            black_box(
                renderer
                    .render(black_box(&scene_100k), black_box(&camera))
                    .unwrap(),
            );
        });
    });
}
