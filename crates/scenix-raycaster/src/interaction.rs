use alloc::vec::Vec;

use scenix_camera::{OrthographicCamera, PerspectiveCamera};
use scenix_core::NodeId;
use scenix_math::{Aabb, Mat4, Plane, Quat, Ray3, Transform, Vec2, Vec3, Vec4};
use scenix_scene::{SceneGraph, SnapSettings, TransformConstraint, TransformMode, TransformSpace};

use crate::{GeometryProvider, Raycaster};

/// Normalized-device-coordinate selection rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectionRect {
    /// Inclusive minimum corner.
    pub min: Vec2,
    /// Inclusive maximum corner.
    pub max: Vec2,
}

impl SelectionRect {
    /// Creates a normalized rectangle from corners in either order.
    pub fn from_ndc(a: Vec2, b: Vec2) -> Self {
        Self {
            min: Vec2::new(a.x.min(b.x).clamp(-1.0, 1.0), a.y.min(b.y).clamp(-1.0, 1.0)),
            max: Vec2::new(a.x.max(b.x).clamp(-1.0, 1.0), a.y.max(b.y).clamp(-1.0, 1.0)),
        }
    }

    /// Returns whether the rectangle has non-zero area.
    pub fn is_empty(self) -> bool {
        self.max.x - self.min.x <= 1.0e-6 || self.max.y - self.min.y <= 1.0e-6
    }

    fn intersects(self, min: Vec2, max: Vec2) -> bool {
        self.min.x <= max.x && self.max.x >= min.x && self.min.y <= max.y && self.max.y >= min.y
    }

    fn contains(self, min: Vec2, max: Vec2) -> bool {
        min.x >= self.min.x && max.x <= self.max.x && min.y >= self.min.y && max.y <= self.max.y
    }
}

/// Whether marquee selection requires full containment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelectionContainment {
    /// Any projected bounds overlap is accepted.
    #[default]
    Intersects,
    /// Projected bounds must be fully contained.
    Contains,
}

/// Camera-specific selection volume represented by a view-projection matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectionFrustum {
    /// Marquee rectangle in NDC.
    pub rect: SelectionRect,
    /// View-projection matrix used to project world bounds.
    pub view_projection: Mat4,
    /// Containment rule.
    pub containment: SelectionContainment,
}

impl SelectionFrustum {
    /// Creates a perspective-camera selection volume.
    pub fn from_perspective(camera: &PerspectiveCamera, rect: SelectionRect) -> Self {
        Self {
            rect,
            view_projection: camera.view_projection(),
            containment: SelectionContainment::Intersects,
        }
    }

    /// Creates an orthographic-camera selection volume.
    pub fn from_orthographic(camera: &OrthographicCamera, rect: SelectionRect) -> Self {
        Self {
            rect,
            view_projection: camera.view_projection(),
            containment: SelectionContainment::Intersects,
        }
    }

    /// Returns this volume with a containment rule.
    pub const fn containment(mut self, containment: SelectionContainment) -> Self {
        self.containment = containment;
        self
    }

    /// Tests conservative projected AABB bounds.
    pub fn intersects_aabb(self, aabb: Aabb) -> bool {
        if self.rect.is_empty() {
            return false;
        }
        let corners = aabb_corners(aabb);
        let mut min = Vec2::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut projected = 0_u8;
        let mut behind = false;
        for corner in corners {
            let clip = self
                .view_projection
                .mul_vec4(Vec4::new(corner.x, corner.y, corner.z, 1.0));
            if clip.w <= 1.0e-6 {
                behind = true;
                continue;
            }
            let point = Vec2::new(clip.x / clip.w, clip.y / clip.w);
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
            projected += 1;
        }
        if projected == 0 {
            return false;
        }
        match self.containment {
            SelectionContainment::Intersects => self.rect.intersects(min, max),
            SelectionContainment::Contains => !behind && self.rect.contains(min, max),
        }
    }
}

impl Raycaster {
    /// Selects eligible mesh nodes intersecting a camera selection volume.
    pub fn select_in_frustum<G: GeometryProvider + ?Sized>(
        &self,
        frustum: SelectionFrustum,
        scene: &SceneGraph,
        geometries: &G,
    ) -> Vec<NodeId> {
        let mut selected = Vec::new();
        self.select_in_frustum_into(frustum, scene, geometries, &mut selected);
        selected
    }

    /// Writes eligible marquee-selected IDs into reusable storage.
    pub fn select_in_frustum_into<G: GeometryProvider + ?Sized>(
        &self,
        frustum: SelectionFrustum,
        scene: &SceneGraph,
        geometries: &G,
        selected: &mut Vec<NodeId>,
    ) {
        selected.clear();
        let layers = self.layers();
        let mut consider = |id| {
            if !scene.is_selectable(id) {
                return;
            }
            let Some(node) = scene.get(id) else {
                return;
            };
            if node.layer & layers == 0 {
                return;
            }
            let scenix_scene::NodeKind::Mesh { mesh_id, .. } = node.kind else {
                return;
            };
            let Some(geometry) = geometries.geometry(mesh_id) else {
                return;
            };
            let world = scene.world_matrix(id).unwrap_or(Mat4::IDENTITY);
            if frustum.intersects_aabb(geometry.aabb().transform(world)) {
                selected.push(id);
            }
        };
        if let Some(bvh) = self.bvh() {
            let broad_phase = frustum.containment(SelectionContainment::Intersects);
            bvh.visit_bounds(&|bounds| broad_phase.intersects_aabb(bounds), &mut consider);
        } else {
            for id in scene.iter_depth_first() {
                consider(id);
            }
        }
        selected.sort_unstable();
        selected.dedup();
    }
}

/// World-space drag plane with robust ray intersection.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DragPlane {
    /// Mathematical plane.
    pub plane: Plane,
}

impl DragPlane {
    /// Creates a plane facing the camera at a world point.
    pub fn camera_facing(point: Vec3, camera_position: Vec3) -> Self {
        Self::from_normal(point, (camera_position - point).normalize())
    }

    /// Creates a plane from a point and normal.
    pub fn from_normal(point: Vec3, normal: Vec3) -> Self {
        Self {
            plane: Plane::from_normal_and_point(normal, point),
        }
    }

    /// Creates a plane normal to a world axis.
    pub fn axis_aligned(point: Vec3, axis: Vec3) -> Self {
        Self::from_normal(point, axis)
    }

    /// Returns the world intersection point.
    pub fn intersect(self, ray: Ray3) -> Option<Vec3> {
        self.plane
            .intersect_ray(ray)
            .map(|distance| ray.at(distance))
    }
}

/// Interaction operation failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InteractionError {
    /// Node does not exist.
    InvalidNode,
    /// Node is locked or filtered by policy.
    Locked,
    /// Ray does not intersect the interaction plane.
    ParallelRay,
    /// No operation is active.
    NotActive,
}

/// Successful transform update.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InteractionUpdate {
    /// Modified node.
    pub node_id: NodeId,
    /// New local transform.
    pub transform: Transform,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ActiveDrag {
    node_id: NodeId,
    plane: DragPlane,
    start_local: Transform,
    start_world_point: Vec3,
    grab_offset: Vec3,
    snap: SnapSettings,
}

/// Reversible plane-based node drag controller.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragController {
    active: Option<ActiveDrag>,
}

impl DragController {
    /// Starts dragging an eligible node.
    pub fn begin(
        &mut self,
        scene: &mut SceneGraph,
        node_id: NodeId,
        ray: Ray3,
        plane: DragPlane,
        snap: SnapSettings,
    ) -> Result<(), InteractionError> {
        let Some(node) = scene.get(node_id) else {
            return Err(InteractionError::InvalidNode);
        };
        if !scene.is_draggable(node_id) {
            return Err(InteractionError::Locked);
        }
        let Some(start_world_point) = plane.intersect(ray) else {
            return Err(InteractionError::ParallelRay);
        };
        let world_origin = scene
            .world_transform(node_id)
            .map_or(node.transform.translation, |transform| {
                transform.translation
            });
        self.active = Some(ActiveDrag {
            node_id,
            plane,
            start_local: node.transform,
            start_world_point,
            grab_offset: world_origin - start_world_point,
            snap,
        });
        scene
            .set_active(Some(node_id))
            .map_err(|_| InteractionError::Locked)
    }

    /// Updates the active drag from a ray.
    pub fn update(
        &mut self,
        scene: &mut SceneGraph,
        ray: Ray3,
    ) -> Result<InteractionUpdate, InteractionError> {
        let Some(active) = self.active else {
            return Err(InteractionError::NotActive);
        };
        let Some(point) = active.plane.intersect(ray) else {
            return Err(InteractionError::ParallelRay);
        };
        let start_world_origin = active.start_world_point + active.grab_offset;
        let target_world_origin = point + active.grab_offset;
        let world_delta = active
            .snap
            .snap_translation(target_world_origin - start_world_origin);
        let local_delta =
            world_delta_to_local(scene, active.node_id, start_world_origin, world_delta);
        let mut transform = active.start_local;
        transform.translation += local_delta;
        scene
            .set_editor_transform(active.node_id, transform)
            .map_err(|_| InteractionError::Locked)?;
        Ok(InteractionUpdate {
            node_id: active.node_id,
            transform,
        })
    }

    /// Commits the drag and clears active state.
    pub fn end(&mut self, scene: &mut SceneGraph) -> Result<(), InteractionError> {
        let Some(active) = self.active.take() else {
            return Err(InteractionError::NotActive);
        };
        if scene.selection().active == Some(active.node_id) {
            let _ = scene.set_active(None);
        }
        Ok(())
    }

    /// Restores the starting transform and clears active state.
    pub fn cancel(&mut self, scene: &mut SceneGraph) -> Result<(), InteractionError> {
        let Some(active) = self.active.take() else {
            return Err(InteractionError::NotActive);
        };
        scene
            .set_local_transform(active.node_id, active.start_local)
            .map_err(|_| InteractionError::InvalidNode)?;
        let _ = scene.set_active(None);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ActiveTransform {
    node_id: NodeId,
    plane: DragPlane,
    start_local: Transform,
    start_world: Transform,
    start_point: Vec3,
    mode: TransformMode,
    space: TransformSpace,
    constraint: TransformConstraint,
    snap: SnapSettings,
}

/// Reversible translate/rotate/scale interaction controller.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransformController {
    active: Option<ActiveTransform>,
}

impl TransformController {
    /// Starts a transform interaction on an eligible node.
    #[allow(clippy::too_many_arguments)]
    pub fn begin(
        &mut self,
        scene: &mut SceneGraph,
        node_id: NodeId,
        ray: Ray3,
        plane: DragPlane,
        mode: TransformMode,
        space: TransformSpace,
        constraint: TransformConstraint,
        snap: SnapSettings,
    ) -> Result<(), InteractionError> {
        let Some(node) = scene.get(node_id) else {
            return Err(InteractionError::InvalidNode);
        };
        if !scene.is_transformable(node_id) {
            return Err(InteractionError::Locked);
        }
        let Some(start_point) = plane.intersect(ray) else {
            return Err(InteractionError::ParallelRay);
        };
        let start_world = scene.world_transform(node_id).unwrap_or(node.transform);
        self.active = Some(ActiveTransform {
            node_id,
            plane,
            start_local: node.transform,
            start_world,
            start_point,
            mode,
            space,
            constraint,
            snap,
        });
        scene
            .set_active(Some(node_id))
            .map_err(|_| InteractionError::Locked)
    }

    /// Applies a ray-derived transform update.
    pub fn update(
        &mut self,
        scene: &mut SceneGraph,
        ray: Ray3,
    ) -> Result<InteractionUpdate, InteractionError> {
        let Some(active) = self.active else {
            return Err(InteractionError::NotActive);
        };
        let Some(point) = active.plane.intersect(ray) else {
            return Err(InteractionError::ParallelRay);
        };
        let mut transform = active.start_local;
        match active.mode {
            TransformMode::Translate => {
                let mut delta = point - active.start_point;
                delta = constrain_vector(
                    delta,
                    active.constraint,
                    active.space,
                    active.start_world.rotation,
                );
                delta = active.snap.snap_translation(delta);
                transform.translation += world_delta_to_local(
                    scene,
                    active.node_id,
                    active.start_world.translation,
                    delta,
                );
            }
            TransformMode::Rotate => {
                let axis = active
                    .constraint
                    .axis()
                    .unwrap_or(active.plane.plane.normal)
                    .normalize();
                let start = (active.start_point - active.start_world.translation).normalize();
                let current = (point - active.start_world.translation).normalize();
                let mut angle = start.angle_between(current);
                angle *= axis.dot(start.cross(current)).signum();
                angle = active.snap.snap_rotation(angle);
                let rotation = Quat::from_axis_angle(axis, angle);
                transform.rotation = match active.space {
                    TransformSpace::World => (rotation * active.start_local.rotation).normalize(),
                    TransformSpace::Local => (active.start_local.rotation * rotation).normalize(),
                };
            }
            TransformMode::Scale => {
                let start_distance = active.start_point.distance(active.start_world.translation);
                let current_distance = point.distance(active.start_world.translation);
                let uniform_delta = if start_distance > 1.0e-6 {
                    current_distance / start_distance - 1.0
                } else {
                    0.0
                };
                let mask = active.constraint.component_mask();
                let delta = active.snap.snap_scale(mask * uniform_delta);
                transform.scale =
                    clamp_scale(active.start_local.scale.mul_elements(Vec3::ONE + delta));
            }
        }
        scene
            .set_editor_transform(active.node_id, transform)
            .map_err(|_| InteractionError::Locked)?;
        Ok(InteractionUpdate {
            node_id: active.node_id,
            transform,
        })
    }

    /// Commits the active transform.
    pub fn end(&mut self, scene: &mut SceneGraph) -> Result<(), InteractionError> {
        let Some(active) = self.active.take() else {
            return Err(InteractionError::NotActive);
        };
        if scene.selection().active == Some(active.node_id) {
            let _ = scene.set_active(None);
        }
        Ok(())
    }

    /// Restores the starting transform.
    pub fn cancel(&mut self, scene: &mut SceneGraph) -> Result<(), InteractionError> {
        let Some(active) = self.active.take() else {
            return Err(InteractionError::NotActive);
        };
        scene
            .set_local_transform(active.node_id, active.start_local)
            .map_err(|_| InteractionError::InvalidNode)?;
        let _ = scene.set_active(None);
        Ok(())
    }
}

fn aabb_corners(aabb: Aabb) -> [Vec3; 8] {
    let min = aabb.min;
    let max = aabb.max;
    [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
    ]
}

fn world_delta_to_local(
    scene: &SceneGraph,
    node_id: NodeId,
    world_origin: Vec3,
    world_delta: Vec3,
) -> Vec3 {
    let parent_inverse = scene
        .parent(node_id)
        .and_then(|parent| scene.world_matrix(parent))
        .and_then(Mat4::inverse)
        .unwrap_or(Mat4::IDENTITY);
    parent_inverse.mul_vec3(world_origin + world_delta) - parent_inverse.mul_vec3(world_origin)
}

fn constrain_vector(
    delta: Vec3,
    constraint: TransformConstraint,
    space: TransformSpace,
    world_rotation: Quat,
) -> Vec3 {
    let mask = constraint.component_mask();
    match space {
        TransformSpace::World => delta.mul_elements(mask),
        TransformSpace::Local => {
            world_rotation.mul_vec3(world_rotation.inverse().mul_vec3(delta).mul_elements(mask))
        }
    }
}

fn clamp_scale(scale: Vec3) -> Vec3 {
    fn lane(value: f32) -> f32 {
        if value.abs() < 1.0e-4 {
            if value < 0.0 { -1.0e-4 } else { 1.0e-4 }
        } else {
            value
        }
    }
    Vec3::new(lane(scale.x), lane(scale.y), lane(scale.z))
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenix_scene::SceneNode;

    #[test]
    fn selection_rect_normalizes_reversed_corners() {
        let rect = SelectionRect::from_ndc(Vec2::new(0.5, 0.7), Vec2::new(-0.5, -0.7));
        assert_eq!(rect.min, Vec2::new(-0.5, -0.7));
        assert_eq!(rect.max, Vec2::new(0.5, 0.7));
    }

    #[test]
    fn drag_cancel_restores_starting_transform() {
        let mut scene = SceneGraph::new();
        let id = scene.add(SceneNode::new("drag"));
        scene.update_world_transforms();
        let plane = DragPlane::from_normal(Vec3::ZERO, Vec3::Z);
        let start_ray = Ray3::new(Vec3::new(0.0, 0.0, 1.0), Vec3::NEG_Z);
        let moved_ray = Ray3::new(Vec3::new(2.0, 0.0, 1.0), Vec3::NEG_Z);
        let mut drag = DragController::default();
        drag.begin(&mut scene, id, start_ray, plane, SnapSettings::default())
            .unwrap();
        drag.update(&mut scene, moved_ray).unwrap();
        assert_eq!(scene.get(id).unwrap().transform.translation.x, 2.0);
        drag.cancel(&mut scene).unwrap();
        assert_eq!(scene.get(id).unwrap().transform, Transform::IDENTITY);
    }

    #[test]
    fn scale_clamp_preserves_negative_sign() {
        assert_eq!(
            clamp_scale(Vec3::new(-0.0, -1.0e-6, 1.0e-6)),
            Vec3::new(1.0e-4, -1.0e-4, 1.0e-4)
        );
    }
}
