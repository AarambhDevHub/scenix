use std::f32::consts::PI;

use scenix_math::{Ray3, Transform, Vec3};

/// World-space length of a gizmo axis handle, from its origin outward.
pub const GIZMO_HALF_LENGTH: f32 = 1.0;

/// Maximum world-space distance between a pointer ray and a handle axis for
/// the axis to count as picked.
pub const GIZMO_PICK_THRESHOLD: f32 = 0.08;

/// Default translation snap step in world units (0 disables snapping).
pub const TRANSLATE_SNAP_STEP: f32 = 0.5;

/// Default rotation snap step in radians (15 degrees).
pub const ROTATE_SNAP_RADIANS: f32 = PI / 12.0;

/// Default scale snap step as a per-touch multiplier delta.
pub const SCALE_SNAP_STEP: f32 = 0.1;

/// Which transform tool is active on a gizmo drag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GizmoMode {
    /// Move the node.
    Translate,
    /// Rotate the node.
    Rotate,
    /// Scale the node.
    Scale,
}

/// Which gizmo handle is being dragged.
///
/// `Screen` covers billboard handles (screen-space rotate/scale); translating
/// along it is host camera math, so the solver leaves translation untouched.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GizmoAxis {
    /// World +X handle.
    X,
    /// World +Y handle.
    Y,
    /// World +Z handle.
    Z,
    /// Billboard handle in the view plane.
    Screen,
}

impl GizmoAxis {
    /// World-space axis direction; [`GizmoAxis::Screen`] has none.
    pub fn direction(self) -> Option<Vec3> {
        match self {
            Self::X => Some(Vec3::X),
            Self::Y => Some(Vec3::Y),
            Self::Z => Some(Vec3::Z),
            Self::Screen => None,
        }
    }
}

/// Snap steps for gizmo application. Zero disables a channel.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GizmoSettings {
    /// Translation snap in world units.
    pub translate_step: f32,
    /// Rotation snap in radians.
    pub rotate_step: f32,
    /// Scale snap as a multiplier delta.
    pub scale_step: f32,
}

impl GizmoSettings {
    /// No snapping — every drag resolution is freeform.
    pub const FREE: Self = Self {
        translate_step: 0.0,
        rotate_step: 0.0,
        scale_step: 0.0,
    };

    /// Default grid snapping: half-unit translates, 15° rotations, 0.1 scale.
    pub const SNAPPED: Self = Self {
        translate_step: TRANSLATE_SNAP_STEP,
        rotate_step: ROTATE_SNAP_RADIANS,
        scale_step: SCALE_SNAP_STEP,
    };

    /// Snap a raw amount onto `step` (`step <= 0.0` disables snapping).
    pub fn snap(&self, value: f32, step: f32) -> f32 {
        if step > 0.0 {
            (value / step).round() * step
        } else {
            value
        }
    }
}

impl Default for GizmoSettings {
    fn default() -> Self {
        Self::SNAPPED
    }
}

/// A resolved gizmo delta, kept as a record so hosts can log or replay one
/// drag step against any transform.
///
/// Each field is zero/identity on lanes the `(mode, axis)` pair does not
/// touch; [`TransformDelta::apply_to`] composes the whole record.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransformDelta {
    /// Snapped world-space translation (translate mode only).
    pub translation: Vec3,
    /// Per-axis rotation amounts in radians, X then Y then Z lanes.
    pub rotation_radians: [f32; 3],
    /// Per-axis scale multipliers.
    pub scale_multiplier: Vec3,
}

impl TransformDelta {
    /// No change.
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation_radians: [0.0; 3],
        scale_multiplier: Vec3::ONE,
    };

    /// Resolve a raw drag amount into a snapped delta for `mode` + `axis`.
    pub fn resolve(
        mode: GizmoMode,
        axis: GizmoAxis,
        amount: f32,
        settings: &GizmoSettings,
    ) -> Self {
        let mut delta = Self::IDENTITY;
        match mode {
            GizmoMode::Translate => {
                if let Some(direction) = axis.direction() {
                    delta.translation = direction * settings.snap(amount, settings.translate_step);
                }
            }
            GizmoMode::Rotate => {
                if let Some(lane) = axis_lane(axis) {
                    delta.rotation_radians[lane] = settings.snap(amount, settings.rotate_step);
                }
            }
            GizmoMode::Scale => {
                let factor = 1.0 + settings.snap(amount, settings.scale_step);
                delta.scale_multiplier = match axis {
                    GizmoAxis::X => Vec3::new(factor, 1.0, 1.0),
                    GizmoAxis::Y => Vec3::new(1.0, factor, 1.0),
                    GizmoAxis::Z => Vec3::new(1.0, 1.0, factor),
                    // Screen scaling resizes every axis uniformly.
                    GizmoAxis::Screen => Vec3::new(factor, factor, factor),
                };
            }
        }
        delta
    }

    /// Compose the delta onto a current transform.
    ///
    /// Rotations apply about world axes (`delta * current`), which is what a
    /// screen-driven gizmo promises: dragging the red ring always tilts about
    /// world X regardless of the node's existing orientation.
    pub fn apply_to(self, current: Transform) -> Transform {
        let [x, y, z] = self.rotation_radians;
        let mut rotation = current.rotation;
        if x != 0.0 {
            rotation = quat_about_x(x) * rotation;
        }
        if y != 0.0 {
            rotation = quat_about_y(y) * rotation;
        }
        if z != 0.0 {
            rotation = quat_about_z(z) * rotation;
        }
        Transform {
            translation: current.translation + self.translation,
            rotation,
            scale: current.scale.mul_elements(self.scale_multiplier),
        }
    }
}

#[inline]
fn axis_lane(axis: GizmoAxis) -> Option<usize> {
    match axis {
        GizmoAxis::X => Some(0),
        GizmoAxis::Y => Some(1),
        GizmoAxis::Z => Some(2),
        GizmoAxis::Screen => None,
    }
}

// Small helpers keep the apply_to arms free of repeated axis math while
// staying allocation free.
#[inline]
fn quat_about_x(radians: f32) -> scenix_math::Quat {
    scenix_math::Quat::from_axis_angle(Vec3::X, radians)
}

#[inline]
fn quat_about_y(radians: f32) -> scenix_math::Quat {
    scenix_math::Quat::from_axis_angle(Vec3::Y, radians)
}

#[inline]
fn quat_about_z(radians: f32) -> scenix_math::Quat {
    scenix_math::Quat::from_axis_angle(Vec3::Z, radians)
}

/// Snap a translation vector componentwise.
///
/// Convenience for hosts that compute whole-plane drags themselves;
/// [`TransformDelta::resolve`] already snaps single-axis amounts.
pub fn snap_translation(delta: Vec3, settings: &GizmoSettings) -> Vec3 {
    Vec3::new(
        settings.snap(delta.x, settings.translate_step),
        settings.snap(delta.y, settings.translate_step),
        settings.snap(delta.z, settings.translate_step),
    )
}

/// Closest-distance gizmo axis picking.
///
/// For each world axis handle (origin to `origin + dir * GIZMO_HALF_LENGTH`)
/// the routine computes the closest approach of `ray` against the handle
/// line; the first axis under [`GIZMO_PICK_THRESHOLD`] with the smallest
/// distance wins. Returns the axis plus the handle parameter `t` (0 to
/// [`GIZMO_HALF_LENGTH`]) where the closest approach happened, so hosts can
/// scale the drag rate by handle length. Allocation free.
pub fn axis_pick(ray: Ray3, origin: Vec3) -> Option<(GizmoAxis, f32)> {
    let mut best: Option<(GizmoAxis, f32, f32)> = None;
    for (axis, direction) in [
        (GizmoAxis::X, Vec3::X),
        (GizmoAxis::Y, Vec3::Y),
        (GizmoAxis::Z, Vec3::Z),
    ] {
        // Standard line-line closest approach for two unit-direction lines.
        let w0 = ray.origin - origin;
        let b = ray.direction.dot(direction);
        let d = ray.direction.dot(w0);
        let e = direction.dot(w0);
        let denom = 1.0 - b * b;
        if denom.abs() < 1.0e-6 {
            continue; // Parallel: the pointer cannot be separating handles.
        }
        let ray_t = (b * e - d) / denom;
        let axis_t = e - b * ray_t;

        // The handle is a finite segment, not an infinite line.
        if !(0.0..=GIZMO_HALF_LENGTH).contains(&axis_t) {
            continue;
        }
        let ray_point = ray.origin + ray.direction * ray_t;
        let axis_point = origin + direction * axis_t;
        let distance = (ray_point - axis_point).length();
        if distance > GIZMO_PICK_THRESHOLD {
            continue;
        }
        if best.is_none_or(|(_, _, best_distance)| distance < best_distance) {
            best = Some((axis, axis_t, distance));
        }
    }
    best.map(|(axis, t, _)| (axis, t))
}

/// Apply one gizmo step to a transform for `mode` + `axis`.
///
/// The session-level convenience over [`TransformDelta::resolve`] plus
/// [`TransformDelta::apply_to`].
pub fn apply_axis_delta(
    current: Transform,
    mode: GizmoMode,
    axis: GizmoAxis,
    amount: f32,
    settings: &GizmoSettings,
) -> Transform {
    TransformDelta::resolve(mode, axis, amount, settings).apply_to(current)
}
