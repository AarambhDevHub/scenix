use scenix_core::{Bounded, MaterialId, MeshId, ValidationError};
use scenix_math::{Mat4, Vec2, Vec3};
use scenix_mesh::{BatchedMesh, Geometry, InstancedMesh, Mesh, box_geometry, plane_geometry};

fn close(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-4, "{a} != {b}");
}

#[test]
fn geometry_validation_catches_bad_attributes_and_indices() {
    let mut geometry = Geometry::with_positions(vec![Vec3::ZERO, Vec3::X, Vec3::Y]);
    geometry.normals.push(Vec3::Z);
    assert_eq!(geometry.validate(), Err(ValidationError::InvalidState));

    geometry.normals.clear();
    geometry.indices = vec![0, 1, 7];
    assert_eq!(geometry.validate(), Err(ValidationError::OutOfRange));
}

#[test]
fn geometry_computes_face_weighted_unit_normals() {
    let mut geometry =
        Geometry::with_positions(vec![Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::new(1.0, 1.0, 0.0)]);
    geometry.indices = vec![0, 1, 2, 1, 3, 2];
    geometry.compute_normals();

    for normal in &geometry.normals {
        close(normal.length(), 1.0);
        assert!(normal.dot(Vec3::Z) > 0.999);
    }
}

#[test]
fn geometry_computes_tangents_with_handedness() {
    let mut geometry = plane_geometry(2.0, 2.0, 1, 1);
    geometry.compute_tangents();

    assert_eq!(geometry.tangents.len(), geometry.positions.len());
    for tangent in &geometry.tangents {
        close(tangent.truncate().length(), 1.0);
        assert!(tangent.w == 1.0 || tangent.w == -1.0);
    }
}

#[test]
fn geometry_merge_offsets_indices_and_fills_missing_attributes() {
    let mut merged = plane_geometry(1.0, 1.0, 1, 1);
    let incoming = Geometry::with_positions(vec![Vec3::ZERO, Vec3::X, Vec3::Y]);

    merged.merge(&incoming);

    assert_eq!(merged.positions.len(), 7);
    assert_eq!(&merged.indices[6..], &[4, 5, 6]);
    assert_eq!(merged.uvs.len(), 7);
    assert_eq!(merged.uvs[6], Vec2::ZERO);
    assert!(merged.validate().is_ok());
}

#[test]
fn geometry_and_mesh_bounds_match_positions() {
    let geometry = box_geometry(2.0, 4.0, 6.0, 1, 1, 1);
    let aabb = geometry.aabb();
    assert_eq!(aabb.min, Vec3::new(-1.0, -2.0, -3.0));
    assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));

    let mesh = Mesh::new(geometry.clone(), MaterialId::new(3));
    assert_eq!(Bounded::aabb(&mesh), aabb);
    assert_eq!(Bounded::aabb(&geometry), aabb);
}

#[test]
fn instanced_mesh_sets_transforms_by_index() {
    let mut instances = InstancedMesh::with_capacity(MeshId::new(1), MaterialId::new(2), 2);
    instances.push_transform(Mat4::IDENTITY);

    let moved = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(instances.set_transform_at(0, moved), Ok(()));
    assert_eq!(instances.transform_at(0), Some(moved));
    assert_eq!(
        instances.set_transform_at(1, Mat4::IDENTITY),
        Err(ValidationError::OutOfRange)
    );
}

#[test]
fn batched_mesh_records_ranges_after_merging() {
    let plane = plane_geometry(1.0, 1.0, 1, 1);
    let cube = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);
    let mut batch = BatchedMesh::new();

    let first = batch.add_geometry(&plane, MaterialId::new(1)).unwrap();
    let second = batch.add_geometry(&cube, MaterialId::new(2)).unwrap();

    assert_eq!(first, 0);
    assert_eq!(second, 1);
    assert_eq!(batch.ranges()[0].vertex_start, 0);
    assert_eq!(batch.ranges()[1].vertex_start, plane.positions.len() as u32);
    assert_eq!(
        batch.geometry.positions.len(),
        plane.positions.len() + cube.positions.len()
    );
    assert!(batch.geometry.validate().is_ok());
}
