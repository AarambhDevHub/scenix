use std::collections::BTreeMap;

use scenix_camera::PerspectiveCamera;
use scenix_core::{MaterialId, MeshId};
use scenix_math::{Ray3, Transform, Vec2, Vec3};
use scenix_mesh::Geometry;
use scenix_raycaster::{Bvh, BvhEntry, Raycaster};
use scenix_scene::{SceneGraph, SceneNode};

fn close(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-4, "{a} != {b}");
}

fn triangle_geometry() -> Geometry {
    Geometry {
        positions: vec![Vec3::ZERO, Vec3::X, Vec3::Y],
        normals: vec![Vec3::Z; 3],
        uvs: vec![Vec2::ZERO, Vec2::X, Vec2::Y],
        ..Geometry::new()
    }
}

fn scene_with_two_triangles() -> (SceneGraph, BTreeMap<MeshId, Geometry>) {
    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(7);
    let mut geometries = BTreeMap::new();
    geometries.insert(mesh_id, triangle_geometry());

    let mut scene = SceneGraph::new();
    scene.add(SceneNode::mesh("near", mesh_id, material_id));
    scene.add(
        SceneNode::mesh("far", mesh_id, material_id)
            .transform(Transform::from_translation(Vec3::new(0.0, 0.0, -2.0))),
    );
    scene.update_world_transforms();
    (scene, geometries)
}

#[test]
fn math_ray_intersections_are_available_for_raycasting() {
    let ray = Ray3::new(Vec3::new(0.25, 0.25, 1.0), Vec3::NEG_Z);
    close(
        ray.intersect_aabb(scenix_math::Aabb::new(Vec3::ZERO, Vec3::ONE))
            .unwrap(),
        0.0,
    );
    let sphere_ray = Ray3::new(Vec3::new(0.0, 0.0, 5.0), Vec3::NEG_Z);
    close(sphere_ray.intersect_sphere(Vec3::ZERO, 1.0).unwrap(), 4.0);
    let (distance, bary) = ray
        .intersect_triangle(Vec3::ZERO, Vec3::X, Vec3::Y)
        .unwrap();
    close(distance, 1.0);
    close(bary.x, 0.25);
    close(bary.y, 0.25);
}

#[test]
fn camera_ndc_center_ray_points_forward() {
    let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 10.0)
        .position(Vec3::new(0.0, 0.0, 5.0))
        .target(Vec3::ZERO);
    let ray = Raycaster::from_camera_ndc(&camera, Vec2::ZERO);
    assert!(ray.direction.z < -0.99);
}

#[test]
fn raycaster_returns_sorted_intersections_with_attributes() {
    let (scene, geometries) = scene_with_two_triangles();
    let mut raycaster = Raycaster::new();
    raycaster.build_bvh(&scene, &geometries).unwrap();

    let ray = Ray3::new(Vec3::new(0.25, 0.25, 1.0), Vec3::NEG_Z);
    let hits = raycaster.cast_ray_all(ray, &scene, &geometries);
    assert_eq!(hits.len(), 2);
    assert!(hits[0].distance < hits[1].distance);
    close(hits[0].distance, 1.0);
    close(hits[0].point.x, 0.25);
    close(hits[0].point.y, 0.25);
    close(hits[0].normal.z, 1.0);
    close(hits[0].uv.x, 0.25);
    close(hits[0].uv.y, 0.25);

    let nearest = raycaster.cast_ray(ray, &scene, &geometries).unwrap();
    assert_eq!(nearest, hits[0]);
}

#[test]
fn bvh_results_match_bruteforce() {
    let (scene, geometries) = scene_with_two_triangles();
    let mut raycaster = Raycaster::new();
    raycaster.build_bvh(&scene, &geometries).unwrap();

    let rays = [
        Ray3::new(Vec3::new(0.1, 0.1, 2.0), Vec3::NEG_Z),
        Ray3::new(Vec3::new(0.8, 0.1, 2.0), Vec3::NEG_Z),
        Ray3::new(Vec3::new(2.0, 2.0, 2.0), Vec3::NEG_Z),
    ];
    for ray in rays {
        let bvh_hits = raycaster.cast_ray_all(ray, &scene, &geometries);
        let brute_hits = raycaster.cast_ray_all_bruteforce(ray, &scene, &geometries);
        assert_eq!(bvh_hits.len(), brute_hits.len());
        for (a, b) in bvh_hits.iter().zip(brute_hits) {
            assert_eq!(a.node_id, b.node_id);
            close(a.distance, b.distance);
        }
    }
}

#[test]
fn layer_and_visibility_filters_exclude_nodes() {
    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(1);
    let mut geometries = BTreeMap::new();
    geometries.insert(mesh_id, triangle_geometry());

    let mut scene = SceneGraph::new();
    let hidden = scene.add(SceneNode::mesh("hidden", mesh_id, material_id).visible(false));
    let layer_two = scene.add(SceneNode::mesh("layer_two", mesh_id, material_id).layer(0b10));
    scene.update_world_transforms();

    let mut raycaster = Raycaster::with_layers(0b01);
    raycaster.build_bvh(&scene, &geometries).unwrap();
    let ray = Ray3::new(Vec3::new(0.25, 0.25, 1.0), Vec3::NEG_Z);
    assert!(raycaster.cast_ray(ray, &scene, &geometries).is_none());

    raycaster.set_layers(0b10);
    raycaster.build_bvh(&scene, &geometries).unwrap();
    let hit = raycaster.cast_ray(ray, &scene, &geometries).unwrap();
    assert_eq!(hit.node_id, layer_two);
    assert_ne!(hit.node_id, hidden);
}

#[test]
fn bvh_traversal_returns_candidate_node_ids() {
    let entries = [
        BvhEntry::new(
            scenix_core::NodeId::new(1),
            scenix_math::Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)),
        ),
        BvhEntry::new(
            scenix_core::NodeId::new(2),
            scenix_math::Aabb::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(6.0, 6.0, 6.0)),
        ),
    ];
    let bvh = Bvh::build(&entries);
    let candidates = bvh.traverse(Ray3::new(Vec3::new(0.0, 0.0, 4.0), Vec3::NEG_Z));
    assert_eq!(candidates, vec![scenix_core::NodeId::new(1)]);
}

#[cfg(feature = "serde")]
#[test]
fn raycaster_types_round_trip_with_serde() {
    let (scene, geometries) = scene_with_two_triangles();
    let mut raycaster = Raycaster::new();
    raycaster.build_bvh(&scene, &geometries).unwrap();
    let json = serde_json::to_string(&raycaster).unwrap();
    let out: Raycaster = serde_json::from_str(&json).unwrap();
    assert_eq!(out.bvh().unwrap().entry_count(), 2);
}
