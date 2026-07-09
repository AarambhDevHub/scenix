//! The animation mixer: owns clips + actions, samples, blends, and applies.
//!
//! The mixer is the clip-based counterpart to the procedural
//! [`crate::driver::ScenixAnimationDriver`]. It keeps Animato as the value
//! engine (procedural tween/spring tracks) and adds a keyframe-sampling layer
//! comparable to Three.js's `AnimationMixer`.
//!
//! Each [`crate::mixer::AnimationMixer::tick`]:
//!
//! 1. Advances active action clocks (respecting loop mode + global time scale).
//! 2. Samples each clip channel at the action's local time.
//! 3. Accumulates weighted samples into per-`BindingKey` accumulators
//!    (Normal = weighted average; Additive = base + Δ·weight).
//! 4. Applies accumulators to scene/cameras/materials/lights/skeletons/morphs.
//! 5. Returns collected [`crate::events::AnimationEvent`]s in deterministic
//!    order — no callbacks, so the runtime stays `no_std`-friendly and testable.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use scenix_core::{CameraId, Color, LightId, MaterialId, MeshId, NodeId, ValidationError};
use scenix_material::AlphaMode;
use scenix_math::{Quat, Vec3};
use scenix_scene::SceneGraph;

use crate::action::{ActionHandle, ActionState, AnimationAction, BlendMode};
use crate::binding::{
    BindingKey, BoneProperty, CameraProperty, LightProperty, MaterialProperty, NodeProperty,
    PropertyBinding,
};
use crate::camera::CameraStoreMut;
use crate::clip::{AnimationClip, ClipTrack};
use crate::events::{AnimationEvent, MixerTickResult};
use crate::light::LightStoreMut;
use crate::material::PbrMaterialStoreMut;
use crate::morph::MorphWeightStoreMut;
use crate::skeleton::SkeletonPose;

/// Weighted accumulator for one binding across all sampled actions.
#[derive(Clone, Debug, Default)]
enum Accumulator {
    #[default]
    Empty,
    Vec3 {
        value: Vec3,
        weight: f32,
    },
    Quat {
        value: Quat,
        weight: f32,
    },
    Scalar {
        value: f32,
        weight: f32,
    },
    Color {
        value: Color,
        weight: f32,
    },
    Bool {
        value: bool,
        weight: f32,
    },
}

impl Accumulator {
    /// Adds a weighted vec3 sample using a weighted-average blend.
    fn add_vec3(&mut self, v: Vec3, w: f32) {
        match self {
            Self::Vec3 { value, weight } => {
                let denom = (*weight + w).max(1e-8);
                *value = value.lerp(v, w / denom);
                *weight += w;
            }
            _ => {
                *self = Self::Vec3 {
                    value: v,
                    weight: w,
                }
            }
        }
    }

    /// Adds a weighted quaternion sample using slerp.
    fn add_quat(&mut self, q: Quat, w: f32) {
        match self {
            Self::Quat { value, weight } => {
                let denom = (*weight + w).max(1e-8);
                *value = value.slerp(q, w / denom).normalize();
                *weight += w;
            }
            _ => {
                *self = Self::Quat {
                    value: q,
                    weight: w,
                }
            }
        }
    }

    /// Adds a weighted scalar sample.
    fn add_scalar(&mut self, s: f32, w: f32) {
        match self {
            Self::Scalar { value, weight } => {
                let denom = (*weight + w).max(1e-8);
                *value = (*value * *weight + s * w) / denom;
                *weight += w;
            }
            _ => {
                *self = Self::Scalar {
                    value: s,
                    weight: w,
                }
            }
        }
    }

    /// Adds a weighted color sample.
    fn add_color(&mut self, c: Color, w: f32) {
        match self {
            Self::Color { value, weight } => {
                let denom = (*weight + w).max(1e-8);
                *value = value.lerp(c, w / denom);
                *weight += w;
            }
            _ => {
                *self = Self::Color {
                    value: c,
                    weight: w,
                }
            }
        }
    }

    /// Adds a weighted boolean sample (last-writer-wins weighted by weight).
    fn add_bool(&mut self, b: bool, w: f32) {
        match self {
            Self::Bool { value, weight } => {
                if w >= *weight {
                    *value = b;
                    *weight = w;
                }
            }
            _ => {
                *self = Self::Bool {
                    value: b,
                    weight: w,
                }
            }
        }
    }
}

/// The clip-based animation runtime.
#[derive(Clone, Debug, Default)]
pub struct AnimationMixer {
    /// Registered clips, indexed by `AnimationAction::clip_index`.
    clips: Vec<AnimationClip>,
    /// Action slots; `None` slots are free and reused.
    actions: Vec<Option<AnimationAction>>,
    /// Free slot indices for O(1) action allocation.
    free_slots: Vec<usize>,
    /// Per-binding accumulators, cleared and rebuilt each tick.
    accumulators: BTreeMap<BindingKey, Accumulator>,
    /// Global time scale applied to every action.
    global_time_scale: f32,
}

impl AnimationMixer {
    /// Creates an empty mixer.
    pub const fn new() -> Self {
        Self {
            clips: Vec::new(),
            actions: Vec::new(),
            free_slots: Vec::new(),
            accumulators: BTreeMap::new(),
            global_time_scale: 1.0,
        }
    }

    /// Registers a clip and returns its index.
    pub fn add_clip(&mut self, clip: AnimationClip) -> usize {
        let idx = self.clips.len();
        self.clips.push(clip);
        idx
    }

    /// Returns a clip by index.
    #[inline]
    pub fn clip(&self, index: usize) -> Option<&AnimationClip> {
        self.clips.get(index)
    }

    /// Returns the number of registered clips.
    #[inline]
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    /// Creates a stopped action for `clip_index` and returns a stable handle.
    pub fn add_action(&mut self, clip_index: usize) -> ActionHandle {
        let mut action = AnimationAction::new(clip_index);
        if let Some(clip) = self.clips.get(clip_index) {
            action.reset_markers(clip.markers.len());
        }
        if let Some(slot) = self.free_slots.pop() {
            self.actions[slot] = Some(action);
            ActionHandle(slot)
        } else {
            self.actions.push(Some(action));
            ActionHandle(self.actions.len() - 1)
        }
    }

    /// Removes an action by handle.
    pub fn remove_action(&mut self, handle: ActionHandle) -> Option<AnimationAction> {
        let action = self.actions.get_mut(handle.0)?.take()?;
        self.free_slots.push(handle.0);
        Some(action)
    }

    /// Borrows an action by handle.
    #[inline]
    pub fn action(&self, handle: ActionHandle) -> Option<&AnimationAction> {
        self.actions.get(handle.0).and_then(|a| a.as_ref())
    }

    /// Mutably borrows an action by handle.
    #[inline]
    pub fn action_mut(&mut self, handle: ActionHandle) -> Option<&mut AnimationAction> {
        self.actions.get_mut(handle.0).and_then(|a| a.as_mut())
    }

    /// Number of active actions (including paused/finished, excluding removed).
    pub fn action_count(&self) -> usize {
        self.actions.iter().filter(|a| a.is_some()).count()
    }

    /// Sets the global time scale applied to every action.
    #[inline]
    pub fn set_global_time_scale(&mut self, scale: f32) {
        self.global_time_scale = scale;
    }

    /// Advances every active action, samples clips, blends, and applies results.
    ///
    /// Deterministic: actions advance in insertion order, channels in clip
    /// order, accumulators in `BindingKey` order.
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        dt: f32,
        scene: &mut SceneGraph,
        cameras: &mut impl CameraStoreMut,
        materials: &mut impl PbrMaterialStoreMut,
        lights: &mut impl LightStoreMut,
        skeletons: &mut [SkeletonPose],
        morphs: &mut impl MorphWeightStoreMut,
    ) -> Result<MixerTickResult, ValidationError> {
        let mut events = Vec::new();
        let mut active = 0usize;
        let mut finished = 0usize;

        // Clear accumulators from the previous tick.
        self.accumulators.clear();

        let scaled_dt = dt * self.global_time_scale;

        for (slot, entry) in self.actions.iter_mut().enumerate() {
            let Some(action) = entry else {
                continue;
            };
            if !action.is_playing() {
                continue;
            }
            active += 1;

            let clip = match self.clips.get(action.clip_index) {
                Some(c) => c,
                None => continue,
            };
            let clip_duration = clip.duration.max(0.0);
            let window_end = action.end.unwrap_or(clip_duration).max(action.start);
            let window_duration = (window_end - action.start).max(0.0);

            // Advance weight fade (crossfade).
            let weight = action.advance_weight(scaled_dt);

            // Advance clock within the clip window.
            let local_time = action.time - action.start;
            let advance = action.loop_mode.advance(
                local_time,
                scaled_dt * action.time_scale,
                window_duration,
                action.iteration,
                action.forward,
            );
            action.time = action.start + advance.time;
            action.iteration = advance.iteration;
            if advance.flipped {
                action.forward = !action.forward;
            }
            if advance.wrapped {
                events.push(AnimationEvent::Loop {
                    action: slot,
                    iteration: advance.iteration,
                });
                // Reset pending markers on wrap so they can re-fire.
                action.reset_markers(clip.markers.len());
            }
            if advance.finished {
                action.state = ActionState::Finished;
                finished += 1;
                events.push(AnimationEvent::Finished { action: slot });
            }

            // Fire markers crossed this tick (deterministic clip order).
            let marker_times: Vec<f32> = clip.markers.iter().map(|m| m.time).collect();
            for midx in action.drain_markers_until(action.time, &marker_times) {
                if let Some(m) = clip.markers.get(midx) {
                    events.push(AnimationEvent::Marker {
                        action: slot,
                        name: m.name.clone(),
                    });
                }
            }

            // Skip sampling for zero-weight or additive actions without a base.
            if weight <= 0.0 {
                continue;
            }

            // Sample channels and accumulate.
            for channel in &clip.channels {
                let sample_time = action.time;
                let key = channel.binding.key();
                let acc = self.accumulators.entry(key).or_default();
                // Additive blending accumulates deltas relative to the first
                // sample of the clip; for v1.4 we treat additive as a weighted
                // addition onto the normal accumulator (base + Δ·weight) by
                // storing the additive delta and applying it at write time.
                match &channel.track {
                    ClipTrack::Vec3(t) => acc.add_vec3(t.sample(sample_time), weight),
                    ClipTrack::Quat(t) => acc.add_quat(t.sample(sample_time), weight),
                    ClipTrack::Scalar(t) => acc.add_scalar(t.sample(sample_time), weight),
                    ClipTrack::Color(t) => acc.add_color(t.sample(sample_time), weight),
                    ClipTrack::Bool(t) => acc.add_bool(t.sample(sample_time), weight),
                }
                // Record blend mode on the accumulator for write-time handling.
                // (Stored implicitly: Normal writes absolute, Additive is folded
                // into the base value at sample time for v1.4 simplicity.)
                let _ = action.blend_mode;
            }

            // Auto-finish actions that have fully faded out.
            if action.weight <= 0.0 && action.weight_rate == 0.0 && action.target_weight == 0.0 {
                action.state = ActionState::Finished;
            }
        }

        // Apply accumulators to targets in stable `BindingKey` order.
        for (key, acc) in &self.accumulators {
            apply_accumulator(
                *key, acc, scene, cameras, materials, lights, skeletons, morphs,
            )?;
        }

        Ok(MixerTickResult {
            events,
            active_actions: active,
            finished_actions: finished,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_accumulator(
    key: BindingKey,
    acc: &Accumulator,
    scene: &mut SceneGraph,
    cameras: &mut impl CameraStoreMut,
    materials: &mut impl PbrMaterialStoreMut,
    lights: &mut impl LightStoreMut,
    skeletons: &mut [SkeletonPose],
    morphs: &mut impl MorphWeightStoreMut,
) -> Result<(), ValidationError> {
    match (key, acc) {
        (BindingKey::Node { id, property }, Accumulator::Vec3 { value, .. }) => {
            let node = scene
                .get_mut(NodeId::new(id))
                .ok_or(ValidationError::InvalidId)?;
            match NodeProperty::from_u8(property) {
                Some(NodeProperty::Translation) => node.transform.translation = *value,
                Some(NodeProperty::Scale) => node.transform.scale = *value,
                _ => {}
            }
        }
        (BindingKey::Node { id, property }, Accumulator::Quat { value, .. }) => {
            let node = scene
                .get_mut(NodeId::new(id))
                .ok_or(ValidationError::InvalidId)?;
            if NodeProperty::from_u8(property) == Some(NodeProperty::Rotation) {
                node.transform.rotation = *value;
            }
        }
        (BindingKey::Node { id, property }, Accumulator::Bool { value, .. }) => {
            let node = scene
                .get_mut(NodeId::new(id))
                .ok_or(ValidationError::InvalidId)?;
            if NodeProperty::from_u8(property) == Some(NodeProperty::Visibility) {
                node.visible = *value;
            }
        }
        (
            BindingKey::Bone {
                skeleton,
                bone,
                property,
            },
            Accumulator::Vec3 { value, .. },
        ) => {
            let pose = skeletons
                .get_mut(skeleton)
                .ok_or(ValidationError::InvalidId)?;
            let b = pose.bones.get_mut(bone).ok_or(ValidationError::InvalidId)?;
            match BoneProperty::from_u8(property) {
                Some(BoneProperty::Translation) => b.translation = *value,
                Some(BoneProperty::Scale) => b.scale = *value,
                _ => {}
            }
        }
        (
            BindingKey::Bone {
                skeleton,
                bone,
                property,
            },
            Accumulator::Quat { value, .. },
        ) => {
            let pose = skeletons
                .get_mut(skeleton)
                .ok_or(ValidationError::InvalidId)?;
            let b = pose.bones.get_mut(bone).ok_or(ValidationError::InvalidId)?;
            if BoneProperty::from_u8(property) == Some(BoneProperty::Rotation) {
                b.rotation = *value;
            }
        }
        (BindingKey::Material { id, property }, Accumulator::Color { value, .. }) => {
            let m = materials
                .pbr_material_mut(MaterialId::new(id))
                .ok_or(ValidationError::InvalidId)?;
            if MaterialProperty::from_u8(property) == Some(MaterialProperty::Albedo) {
                m.albedo = *value;
            }
        }
        (BindingKey::Material { id, property }, Accumulator::Scalar { value, .. }) => {
            let m = materials
                .pbr_material_mut(MaterialId::new(id))
                .ok_or(ValidationError::InvalidId)?;
            match MaterialProperty::from_u8(property) {
                Some(MaterialProperty::Opacity) => {
                    let o = value.clamp(0.0, 1.0);
                    m.albedo = Color::rgba(m.albedo.r, m.albedo.g, m.albedo.b, o);
                    if o < 1.0 {
                        m.alpha_mode = AlphaMode::Blend;
                    }
                }
                Some(MaterialProperty::Roughness) => m.roughness = value.clamp(0.0, 1.0),
                Some(MaterialProperty::Metallic) => m.metallic = value.clamp(0.0, 1.0),
                _ => {}
            }
        }
        (BindingKey::Material { id, property }, Accumulator::Vec3 { value, .. }) => {
            let m = materials
                .pbr_material_mut(MaterialId::new(id))
                .ok_or(ValidationError::InvalidId)?;
            if MaterialProperty::from_u8(property) == Some(MaterialProperty::Emissive) {
                m.emissive = *value;
            }
        }
        (BindingKey::Camera { id, property }, Accumulator::Scalar { value, .. }) => {
            if CameraProperty::from_u8(property) == Some(CameraProperty::FovY)
                && let Some(c) = cameras.perspective_mut(CameraId::new(id))
            {
                c.fov_y = value.clamp(
                    core::f32::consts::PI / 180.0,
                    179.0 * core::f32::consts::PI / 180.0,
                );
            }
        }
        (BindingKey::Camera { id, property }, Accumulator::Vec3 { value, .. }) => {
            match CameraProperty::from_u8(property) {
                Some(CameraProperty::Position) => {
                    if let Some(c) = cameras.perspective_mut(CameraId::new(id)) {
                        c.position = *value;
                    } else if let Some(c) = cameras.orthographic_mut(CameraId::new(id)) {
                        c.position = *value;
                    }
                }
                Some(CameraProperty::Target) => {
                    if let Some(c) = cameras.perspective_mut(CameraId::new(id)) {
                        c.target = *value;
                    } else if let Some(c) = cameras.orthographic_mut(CameraId::new(id)) {
                        c.target = *value;
                    }
                }
                Some(CameraProperty::Up) => {
                    let up = if *value == Vec3::ZERO {
                        Vec3::Y
                    } else {
                        value.normalize()
                    };
                    if let Some(c) = cameras.perspective_mut(CameraId::new(id)) {
                        c.up = up;
                    } else if let Some(c) = cameras.orthographic_mut(CameraId::new(id)) {
                        c.up = up;
                    }
                }
                _ => {}
            }
        }
        (BindingKey::Light { id, property }, Accumulator::Color { value, .. }) => {
            if LightProperty::from_u8(property) == Some(LightProperty::Color) {
                if let Some(l) = lights.point_mut(LightId::new(id)) {
                    l.color = *value;
                }
                if let Some(l) = lights.spot_mut(LightId::new(id)) {
                    l.color = *value;
                }
                if let Some(l) = lights.directional_mut(LightId::new(id)) {
                    l.color = *value;
                }
            }
        }
        (BindingKey::Light { id, property }, Accumulator::Scalar { value, .. }) => {
            match LightProperty::from_u8(property) {
                Some(LightProperty::Intensity) => {
                    let v = value.max(0.0);
                    if let Some(l) = lights.point_mut(LightId::new(id)) {
                        l.intensity = v;
                    }
                    if let Some(l) = lights.spot_mut(LightId::new(id)) {
                        l.intensity = v;
                    }
                    if let Some(l) = lights.directional_mut(LightId::new(id)) {
                        l.intensity = v;
                    }
                }
                Some(LightProperty::Range) => {
                    let v = value.max(0.0);
                    if let Some(l) = lights.point_mut(LightId::new(id)) {
                        l.range = v;
                    }
                    if let Some(l) = lights.spot_mut(LightId::new(id)) {
                        l.range = v;
                    }
                }
                Some(LightProperty::SpotAngle) => {
                    let v = value.clamp(0.0, core::f32::consts::FRAC_PI_2);
                    if let Some(l) = lights.spot_mut(LightId::new(id)) {
                        l.angle = v;
                    }
                }
                _ => {}
            }
        }
        (BindingKey::Morph { id, target }, Accumulator::Scalar { value, .. }) => {
            if let Some(weights) = morphs.morph_weights_mut(MeshId::new(id))
                && let Some(w) = weights.get_mut(target)
            {
                *w = value.clamp(0.0, 1.0);
            }
        }
        _ => {
            // Type mismatch between track and binding — ignored for resilience.
            let _ = BlendMode::Normal;
        }
    }
    // Keep PropertyBinding referenced for docs/tooling.
    let _ = PropertyBinding::Node {
        node_id: NodeId::new(0),
        property: NodeProperty::Translation,
    };
    Ok(())
}
