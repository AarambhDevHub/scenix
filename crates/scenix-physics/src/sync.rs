//! Per-frame node transform synchronization from physics worlds.

use alloc::vec::Vec;

use scenix_core::PhysicsBodyId;
use scenix_math::Transform;

/// One body → node writeback decision.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SyncEntry {
    /// Body whose transform was written.
    pub body: PhysicsBodyId,
    /// Transform the node should adopt this frame.
    pub transform: Transform,
    /// Debug layer marker for engine tooling.
    pub layer: u8,
}

/// Per-frame writeback queue produced by the engine loop.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SyncQueue {
    entries: Vec<SyncEntry>,
}

impl SyncQueue {
    /// Appends one entry.
    #[inline]
    pub fn push(&mut self, entry: SyncEntry) {
        self.entries.push(entry);
    }

    /// Number of dirty nodes this frame.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when no node needs a write.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries in body-id order for deterministic scene writes.
    #[must_use]
    pub fn sorted(&self) -> Vec<SyncEntry> {
        let mut entries = self.entries.clone();
        entries.sort_unstable_by_key(|entry| entry.body.get());
        entries
    }

    /// Iterates the raw entry order.
    #[inline]
    #[must_use]
    pub fn entries(&self) -> &[SyncEntry] {
        &self.entries
    }

    /// Clears the queue for the next frame.
    #[inline]
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Durable body → node mapping kept across frames.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicsTransformSync {
    bindings: Vec<(PhysicsBodyId, u64)>,
}

impl PhysicsTransformSync {
    /// Creates an empty mapping.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds a body to a scene node (opaque `u64` node handle).
    #[inline]
    #[must_use]
    pub fn bind(&mut self, body: PhysicsBodyId, node: u64) -> bool {
        if self.bindings.iter().any(|(b, _)| *b == body) {
            return false;
        }
        self.bindings.push((body, node));
        true
    }

    /// Unbinds a body.
    #[inline]
    pub fn unbind(&mut self, body: PhysicsBodyId) -> bool {
        let before = self.bindings.len();
        self.bindings.retain(|(b, _)| *b != body);
        self.bindings.len() != before
    }

    /// Live binding count.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// True when nothing is bound yet.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Snapshot the queue once: every bound body produces a `SyncEntry`
    /// with its freshest transform. Static-bodies are skipped to keep the
    /// write list small.
    #[must_use]
    pub fn collect(&self, world: &crate::world::PhysicsWorld) -> SyncQueue {
        let mut queue = SyncQueue::default();
        for (body_id, node) in self.bindings.iter() {
            let Some(body) = world.body(*body_id) else {
                continue;
            };
            if body.kind.can_be_pushed() {
                queue.push(SyncEntry {
                    body: *body_id,
                    transform: body.transform,
                    layer: (*node & 0xFF) as u8,
                });
            }
        }
        queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bodies::{PhysicsBody, RigidBodyKind};
    use crate::world::PhysicsWorld;
    use scenix_math::Vec3;

    fn demo_world() -> PhysicsWorld {
        let mut world = PhysicsWorld::earth();
        let ball = PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 5.0, 0.0),
            scenix_math::Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic);
        let id = world.add_body(ball);
        let _ball_collider =
            world.attach_collider(id, crate::colliders::ColliderShape::sphere(0.5));
        let anchor = PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static);
        let static_id = world.add_body(anchor);
        let _floor_collider = world.attach_collider(
            static_id,
            crate::colliders::ColliderShape::boxy(4.0, 0.5, 4.0),
        );
        world
    }

    #[test]
    fn bindings_are_one_to_one() {
        let mut sync = PhysicsTransformSync::new();
        assert!(sync.bind(PhysicsBodyId::new(1), 7));
        assert!(!sync.bind(PhysicsBodyId::new(1), 8));
        assert_eq!(sync.len(), 1);
        assert!(sync.unbind(PhysicsBodyId::new(1)));
        assert!(sync.is_empty());
    }

    #[test]
    fn collection_skips_statics_and_sorts() {
        let world = demo_world();
        let mut sync = PhysicsTransformSync::new();
        let dynamic = PhysicsBodyId::new(1);
        let static_id = PhysicsBodyId::new(2);
        assert!(sync.bind(static_id, 0x2));
        assert!(sync.bind(dynamic, 0x1));
        let queue = sync.collect(&world);
        assert_eq!(queue.len(), 1, "statics are skipped");
        assert_eq!(queue.entries()[0].body, dynamic);
        assert_eq!(queue.sorted().first().copied().map(|e| e.layer), Some(0x1));
    }

    #[test]
    fn queued_entries_clear() {
        let mut queue = SyncQueue::default();
        queue.push(SyncEntry {
            body: PhysicsBodyId::new(3),
            transform: Transform::IDENTITY,
            layer: 0,
        });
        assert_eq!(queue.len(), 1);
        queue.clear();
        assert!(queue.is_empty());
    }
}
