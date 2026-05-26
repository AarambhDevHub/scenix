use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use scenix_core::{MaterialId, MeshId};
use scenix_math::{Ray3, Transform, Vec3};
use scenix_mesh::{Geometry, box_geometry};
use scenix_raycaster::Raycaster;
use scenix_scene::{SceneGraph, SceneNode};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn scene(count: u32) -> (SceneGraph, BTreeMap<MeshId, Geometry>) {
    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(1);
    let mut meshes = BTreeMap::new();
    meshes.insert(mesh_id, box_geometry(1.0, 1.0, 1.0, 1, 1, 1));

    let mut scene = SceneGraph::with_capacity(count as usize);
    for index in 0..count {
        let x = (index % 32) as f32 * 2.0;
        let z = (index / 32) as f32 * -2.0;
        scene.add(
            SceneNode::mesh("cube", mesh_id, material_id)
                .transform(Transform::from_translation(Vec3::new(x, 0.0, z))),
        );
    }
    scene.update_world_transforms();
    (scene, meshes)
}

fn main() {
    let (scene, meshes) = scene(1_024);
    bench("bvh_build_1k", 100, || {
        let mut raycaster = Raycaster::new();
        black_box(
            raycaster
                .build_bvh(black_box(&scene), black_box(&meshes))
                .unwrap(),
        );
    });

    let mut raycaster = Raycaster::new();
    raycaster.build_bvh(&scene, &meshes).unwrap();
    let rays: Vec<_> = (0..1_024)
        .map(|index| Ray3::new(Vec3::new((index % 32) as f32 * 2.0, 0.0, 10.0), Vec3::NEG_Z))
        .collect();
    bench("bvh_query_1k_rays", 100, || {
        for ray in &rays {
            black_box(raycaster.cast_ray(black_box(*ray), black_box(&scene), black_box(&meshes)));
        }
    });
}
