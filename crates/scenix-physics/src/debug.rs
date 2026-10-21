//! Collider wireframe segments for the debug line pipeline.
//!
//! Produces raw `Vec3` vertex pairs so `scenix-helpers` can stream them
//! into its line builder without scenix-physics depending on the helpers
//! crate (the dependency points the other way).

use alloc::vec::Vec;

use scenix_math::{Transform, Vec3};

use crate::colliders::ColliderShape;
use crate::world::PhysicsWorld;

/// Ring segments per full turn for circles in wireframes.
pub const WIREFRAME_SEGMENTS: usize = 12;

/// One wireframe segment: two endpoints.
pub type WireframeSeg = (Vec3, Vec3);

/// Wireframe of one collider shape under a transform.
#[must_use]
pub fn debug_wireframe(shape: &ColliderShape, transform: &Transform) -> Vec<WireframeSeg> {
    match shape {
        ColliderShape::Sphere(radius) => wireframe_sphere(transform.translation, *radius),
        ColliderShape::Box(half) => wireframe_box(transform.translation, *half),
        ColliderShape::Capsule {
            half_height,
            radius,
        } => wireframe_capsule(transform, *half_height, *radius),
        ColliderShape::Plane => {
            // Local tile-sized wireframe; planes stay infinite in queries.
            grid_square(transform.translation, 2.0)
        }
    }
}

/// Wireframes for every collider in a world, collider-id tagged.
#[must_use]
pub fn debug_wireframe_world(
    world: &PhysicsWorld,
) -> Vec<(scenix_core::ColliderId, Vec<WireframeSeg>)> {
    let mut tagged = Vec::new();
    for collider in world.colliders() {
        if let Some(body) = world.body(collider.body_id) {
            tagged.push((
                collider.id,
                debug_wireframe(&collider.shape, &body.transform),
            ));
        }
    }
    tagged
}

/// Closed planar ring: `to_point` receives raw offsets `(dx, dy)` and
/// places the world position.
fn closed_ring(to_point: &dyn Fn(f32, f32) -> Vec3, segments: usize) -> Vec<WireframeSeg> {
    let span = core::f32::consts::PI * 2.0 / segments as f32;
    let mut ring = Vec::with_capacity(segments);
    for step in 0..segments {
        let a = (step as f32) * span;
        let b = ((step + 1) as f32) * span;
        let pa = to_point(radius_offset_x(a), radius_offset_y(a));
        let pb = to_point(radius_offset_x(b), radius_offset_y(b));
        ring.push((pa, pb));
    }
    ring
}

/// `cos(angle) * radius` with the signed offset passed to ring callers.
#[inline]
#[must_use]
fn radius_offset_x(angle: f32) -> f32 {
    scenix_math::float::cos(angle) * 1.0
}

/// `sin(angle) * radius` with the signed offset passed to ring callers.
#[inline]
#[must_use]
fn radius_offset_y(angle: f32) -> f32 {
    scenix_math::float::sin(angle) * 1.0
}

/// Sphere wireframe: equator plus two meridians.
#[must_use]
pub fn wireframe_sphere(center: Vec3, radius: f32) -> Vec<WireframeSeg> {
    let mut segs = Vec::new();
    let equator = |dx: f32, dy: f32| center + Vec3::new(dx * radius, 0.0, dy * radius);
    let xy_ring = |dx: f32, dy: f32| center + Vec3::new(dx * radius, dy * radius, 0.0);
    let yz_ring = |dx: f32, dy: f32| center + Vec3::new(0.0, dy * radius, dx * radius);
    segs.extend(closed_ring(&equator, WIREFRAME_SEGMENTS));
    segs.extend(closed_ring(&xy_ring, WIREFRAME_SEGMENTS));
    segs.extend(closed_ring(&yz_ring, WIREFRAME_SEGMENTS));
    segs
}

/// Wireframe of an axis-aligned box: 12 edges.
#[must_use]
pub fn wireframe_box(center: Vec3, half: Vec3) -> Vec<WireframeSeg> {
    let corners = [
        Vec3::new(-half.x, -half.y, -half.z),
        Vec3::new(half.x, -half.y, -half.z),
        Vec3::new(half.x, -half.y, half.z),
        Vec3::new(-half.x, -half.y, half.z),
        Vec3::new(-half.x, half.y, -half.z),
        Vec3::new(half.x, half.y, -half.z),
        Vec3::new(half.x, half.y, half.z),
        Vec3::new(-half.x, half.y, half.z),
    ];
    let edge_pairs = [
        (0u8, 1u8),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    let to_world = |index: u8| center + corners[index as usize];
    edge_pairs
        .iter()
        .map(|(a, b)| (to_world(*a), to_world(*b)))
        .collect()
}

/// Capsule wireframe: two cap rings plus four shaft rails.
#[must_use]
pub fn wireframe_capsule(
    transform: &Transform,
    half_height: f32,
    radius: f32,
) -> Vec<WireframeSeg> {
    let mut segs = Vec::new();
    let axis = transform.rotation * Vec3::Y;
    let center = transform.translation;
    let (bottom, top) = (center - axis * half_height, center + axis * half_height);
    let (side1, side2) = cap_basis(axis);

    let bottom_ring = |dx: f32, dy: f32| bottom + side1 * (dx * radius) + side2 * (dy * radius);
    let top_ring = |dx: f32, dy: f32| top + side1 * (dx * radius) + side2 * (dy * radius);
    segs.extend(closed_ring(&bottom_ring, WIREFRAME_SEGMENTS));
    segs.extend(closed_ring(&top_ring, WIREFRAME_SEGMENTS));

    // Shaft rails connect the rings at quarter phases.
    for phase in 0u32..4 {
        let angle = phase as f32 * (core::f32::consts::PI * 0.5);
        let offset = side1 * (radius * scenix_math::float::cos(angle))
            + side2 * (radius * scenix_math::float::sin(angle));
        segs.push((bottom + offset, top + offset));
    }
    segs
}

/// Plane grid wireframe: one square of `size` meters around a center.
#[must_use]
fn grid_square(center: Vec3, size: f32) -> Vec<WireframeSeg> {
    let half = size * 0.5;
    let y = center.y;
    let corners = [
        Vec3::new(center.x - half, y, center.z - half),
        Vec3::new(center.x + half, y, center.z - half),
        Vec3::new(center.x + half, y, center.z + half),
        Vec3::new(center.x - half, y, center.z + half),
    ];
    alloc::vec![
        (corners[0], corners[1]),
        (corners[1], corners[2]),
        (corners[2], corners[3]),
        (corners[3], corners[0]),
    ]
}

/// Two side axes orthogonal to the capsule axis.
#[must_use]
fn cap_basis(axis: Vec3) -> (Vec3, Vec3) {
    let helper = if axis.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
    let side1 = axis.cross(helper).normalize();
    let side2 = axis.cross(side1).normalize();
    (side1, side2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenix_math::Quat;

    #[test]
    fn box_wireframes_have_12_edges() {
        let segs = wireframe_box(Vec3::ZERO, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(segs.len(), 12);
        for (a, b) in segs.iter().take(4) {
            assert!((a.y - b.y).abs() < 1.0e-6, "bottom loop is flat");
            assert_eq!(a.y, -2.0);
        }
    }

    #[test]
    fn sphere_wireframes_cover_all_planes() {
        let center = Vec3::new(5.0, 1.0, -2.0);
        let segs = wireframe_sphere(center, 0.7);
        assert_eq!(segs.len(), WIREFRAME_SEGMENTS * 3);
        // Equator endpoints sit at center height, ring XZ radius exact.
        for (a, b) in segs.iter().take(WIREFRAME_SEGMENTS) {
            assert!((a.y - center.y).abs() < 1.0e-5);
            assert!((b.y - center.y).abs() < 1.0e-5);
            let dx = a.x - center.x;
            let dz = a.z - center.z;
            let ring = (dx * dx + dz * dz).sqrt();
            assert!((ring - 0.7).abs() < 0.02, "ring radius={ring}");
        }
    }

    #[test]
    fn capsule_wireframes_have_caps_and_rails() {
        let transform = Transform::new(Vec3::new(0.0, 1.0, 0.0), Quat::IDENTITY, Vec3::ONE);
        let segs = wireframe_capsule(&transform, 0.5, 0.3);
        assert_eq!(
            segs.len(),
            2 * WIREFRAME_SEGMENTS + 4,
            "two rings plus shaft rails"
        );
        let rails = segs.last().copied().expect("rails exist");
        assert!(
            (rails.1.y - rails.0.y).abs() > 0.5,
            "rails run along the axis"
        );
        for (a, b) in segs.iter().take(WIREFRAME_SEGMENTS) {
            assert!((a.y - b.y).abs() < 1.0e-6, "bottom ring sits flat");
        }
    }

    #[test]
    fn world_wireframes_tag_every_collider() {
        let mut world = PhysicsWorld::earth();
        let body = world.add_body(
            crate::bodies::PhysicsBody::new(Transform::IDENTITY)
                .with_kind(crate::bodies::RigidBodyKind::Dynamic),
        );
        let _collider = world.attach_collider(body, ColliderShape::sphere(0.6));
        let tagged = debug_wireframe_world(&world);
        assert_eq!(tagged.len(), 1);
        assert!(!tagged[0].1.is_empty());
    }
}
