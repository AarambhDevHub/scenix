use scenix_math::{Vec2, Vec3};
use scenix_mesh::{
    Shape, box_geometry, capsule_geometry, circle_geometry, cone_geometry, cylinder_geometry,
    extrude_geometry, icosphere_geometry, lathe_geometry, plane_geometry, ring_geometry,
    shape_geometry, sphere_geometry, torus_geometry, torus_knot_geometry, tube_geometry,
};

fn representative_primitives() -> Vec<(&'static str, scenix_mesh::Geometry)> {
    let square = Shape::new(vec![
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, 1.0),
    ]);
    vec![
        ("box", box_geometry(1.0, 1.0, 1.0, 1, 1, 1)),
        ("sphere", sphere_geometry(1.0, 8, 4)),
        ("plane", plane_geometry(1.0, 1.0, 2, 2)),
        ("cylinder", cylinder_geometry(0.5, 0.5, 1.0, 8, 2, false)),
        ("cone", cone_geometry(0.5, 1.0, 8, 2)),
        ("capsule", capsule_geometry(0.25, 1.5, 3, 8)),
        ("torus", torus_geometry(1.0, 0.25, 8, 12)),
        ("torus_knot", torus_knot_geometry(1.0, 0.1, 24, 6, 2, 3)),
        ("icosphere", icosphere_geometry(1.0, 1)),
        (
            "circle",
            circle_geometry(1.0, 16, 0.0, core::f32::consts::TAU),
        ),
        ("ring", ring_geometry(0.4, 1.0, 16, 2)),
        (
            "lathe",
            lathe_geometry(
                &[
                    Vec2::new(0.2, -0.5),
                    Vec2::new(0.4, 0.0),
                    Vec2::new(0.2, 0.5),
                ],
                12,
                0.0,
                core::f32::consts::TAU,
            ),
        ),
        ("extrude", extrude_geometry(&square, 0.5, 0.0, 0.0, 0)),
        (
            "tube",
            tube_geometry(
                &[
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                ],
                8,
                0.1,
                6,
                false,
            ),
        ),
        ("shape", shape_geometry(&square)),
    ]
}

#[test]
fn every_primitive_validates_and_has_uvs_in_range() {
    for (name, geometry) in representative_primitives() {
        assert!(geometry.validate().is_ok(), "{name} did not validate");
        assert!(
            !geometry.positions.is_empty(),
            "{name} produced no positions"
        );
        assert_eq!(
            geometry.uvs.len(),
            geometry.positions.len(),
            "{name} missing uvs"
        );
        for uv in &geometry.uvs {
            assert!(
                uv.x >= -1.0e-5 && uv.x <= 1.0 + 1.0e-5 && uv.y >= -1.0e-5 && uv.y <= 1.0 + 1.0e-5,
                "{name} uv out of range: {uv:?}"
            );
        }
    }
}

#[test]
fn primitive_normals_face_triangle_winding() {
    for (name, geometry) in representative_primitives() {
        for triangle in geometry.indices.chunks_exact(3) {
            let a = triangle[0] as usize;
            let b = triangle[1] as usize;
            let c = triangle[2] as usize;
            let face = (geometry.positions[b] - geometry.positions[a])
                .cross(geometry.positions[c] - geometry.positions[a])
                .normalize();
            if face.length_squared() <= 1.0e-6 {
                continue;
            }
            if geometry.normals.len() == geometry.positions.len() {
                assert!(
                    geometry.normals[a].dot(face) > -1.0e-4
                        && geometry.normals[b].dot(face) > -1.0e-4
                        && geometry.normals[c].dot(face) > -1.0e-4,
                    "{name} normal opposes winding"
                );
            }
        }
    }
}

#[test]
fn deterministic_primitive_counts_match_expected_values() {
    let plane = plane_geometry(1.0, 1.0, 1, 1);
    assert_eq!(plane.positions.len(), 4);
    assert_eq!(plane.indices.len(), 6);

    let cube = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);
    assert_eq!(cube.positions.len(), 24);
    assert_eq!(cube.indices.len(), 36);

    let sphere = sphere_geometry(1.0, 3, 2);
    assert_eq!(sphere.positions.len(), 12);
    assert_eq!(sphere.indices.len(), 36);
}

#[test]
fn degenerate_primitive_inputs_return_empty_geometry() {
    assert!(box_geometry(0.0, 1.0, 1.0, 1, 1, 1).is_empty());
    assert!(sphere_geometry(-1.0, 8, 4).is_empty());
    assert!(cylinder_geometry(0.0, 0.0, 1.0, 8, 1, false).is_empty());
    assert!(torus_geometry(1.0, 0.0, 8, 8).is_empty());
    assert!(tube_geometry(&[Vec3::ZERO], 4, 0.1, 6, false).is_empty());
    assert!(Shape::new(vec![Vec2::ZERO, Vec2::X]).is_empty());
}
