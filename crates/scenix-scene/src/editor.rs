use alloc::{string::String, vec::Vec};

use scenix_core::NodeId;
use scenix_math::Vec3;

/// A bit mask used to filter scene interaction layers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LayerMask(u32);

impl LayerMask {
    /// No layers.
    pub const NONE: Self = Self(0);
    /// Every layer bit.
    pub const ALL: Self = Self(u32::MAX);

    /// Creates a mask from raw bits.
    #[inline]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Returns raw mask bits.
    #[inline]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns whether any bits overlap.
    #[inline]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Returns whether this mask includes the node's layer mask.
    #[inline]
    pub const fn matches_node(self, node_layers: u32) -> bool {
        self.0 & node_layers != 0
    }

    /// Adds bits to the mask.
    #[inline]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Removes bits from the mask.
    #[inline]
    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

/// Layer filters applied by editor interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LayerPolicy {
    /// Layers eligible for selection.
    pub selectable: LayerMask,
    /// Layers eligible for dragging.
    pub draggable: LayerMask,
    /// Layers eligible for transform tools.
    pub transformable: LayerMask,
}

impl Default for LayerPolicy {
    fn default() -> Self {
        Self {
            selectable: LayerMask::ALL,
            draggable: LayerMask::ALL,
            transformable: LayerMask::ALL,
        }
    }
}

/// Sparse editor-only metadata associated with a scene node.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeEditorMetadata {
    /// Node can be selected.
    pub selectable: bool,
    /// Node can be moved by drag tools.
    pub draggable: bool,
    /// Node can be modified by transform tools.
    pub transformable: bool,
    /// Node is protected from editor mutations.
    pub locked: bool,
    /// Node appears in inspector snapshots.
    pub visible_in_inspector: bool,
    /// Optional editor label overriding the scene name.
    pub label: Option<String>,
    /// Application-defined editor tags.
    pub tags: Vec<String>,
}

impl Default for NodeEditorMetadata {
    fn default() -> Self {
        Self {
            selectable: true,
            draggable: true,
            transformable: true,
            locked: false,
            visible_in_inspector: true,
            label: None,
            tags: Vec::new(),
        }
    }
}

/// How a selection command combines with the existing selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelectionMode {
    /// Replace the complete selection.
    #[default]
    Replace,
    /// Add the node if absent.
    Add,
    /// Toggle membership.
    Toggle,
    /// Remove the node if present.
    Remove,
}

/// Current graph-local editor selection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectionState {
    /// Node under the pointer, if any.
    pub hovered: Option<NodeId>,
    /// Node receiving the active interaction, if any.
    pub active: Option<NodeId>,
    selected: Vec<NodeId>,
}

impl SelectionState {
    /// Selected IDs in ascending deterministic order.
    #[inline]
    pub fn selected(&self) -> &[NodeId] {
        &self.selected
    }

    /// Returns whether a node is selected.
    #[inline]
    pub fn contains(&self, id: NodeId) -> bool {
        self.selected.binary_search(&id).is_ok()
    }
}

/// Exact selection differences produced by one operation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectionChange {
    /// IDs added by the operation.
    pub added: Vec<NodeId>,
    /// IDs removed by the operation.
    pub removed: Vec<NodeId>,
}

/// Active transform tool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransformMode {
    /// Move nodes.
    #[default]
    Translate,
    /// Rotate nodes.
    Rotate,
    /// Scale nodes.
    Scale,
}

/// Coordinate system used by a transform tool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransformSpace {
    /// World axes.
    #[default]
    World,
    /// Node-local axes.
    Local,
}

/// Axis or plane constraint used by a transform operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransformConstraint {
    /// No axis constraint.
    #[default]
    Free,
    /// X axis.
    X,
    /// Y axis.
    Y,
    /// Z axis.
    Z,
    /// XY plane.
    XY,
    /// XZ plane.
    XZ,
    /// YZ plane.
    YZ,
}

impl TransformConstraint {
    /// Returns a component mask for translation and scale constraints.
    pub const fn component_mask(self) -> Vec3 {
        match self {
            Self::Free => Vec3::ONE,
            Self::X => Vec3::X,
            Self::Y => Vec3::Y,
            Self::Z => Vec3::Z,
            Self::XY => Vec3::new(1.0, 1.0, 0.0),
            Self::XZ => Vec3::new(1.0, 0.0, 1.0),
            Self::YZ => Vec3::new(0.0, 1.0, 1.0),
        }
    }

    /// Returns the single constrained axis, when applicable.
    pub const fn axis(self) -> Option<Vec3> {
        match self {
            Self::X => Some(Vec3::X),
            Self::Y => Some(Vec3::Y),
            Self::Z => Some(Vec3::Z),
            _ => None,
        }
    }
}

/// Quantization increments for editor transforms. Zero disables a component.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SnapSettings {
    /// Translation increments per axis.
    pub translation: Vec3,
    /// Rotation increment in radians.
    pub rotation_radians: f32,
    /// Scale increments per axis.
    pub scale: Vec3,
}

impl SnapSettings {
    /// Quantizes a translation relative to the operation start.
    pub fn snap_translation(self, delta: Vec3) -> Vec3 {
        snap_vec3(delta, self.translation)
    }

    /// Quantizes a rotation angle relative to the operation start.
    pub fn snap_rotation(self, radians: f32) -> f32 {
        snap_scalar(radians, self.rotation_radians)
    }

    /// Quantizes a scale delta relative to the operation start.
    pub fn snap_scale(self, delta: Vec3) -> Vec3 {
        snap_vec3(delta, self.scale)
    }
}

fn snap_scalar(value: f32, increment: f32) -> f32 {
    if increment.is_finite() && increment.abs() > 1.0e-6 {
        let increment = increment.abs();
        let scaled = value / increment;
        if !scaled.is_finite() {
            return value;
        }
        let rounded = if scaled >= 0.0 {
            (scaled + 0.5) as i64
        } else {
            (scaled - 0.5) as i64
        };
        rounded as f32 * increment
    } else {
        value
    }
}

fn snap_vec3(value: Vec3, increments: Vec3) -> Vec3 {
    Vec3::new(
        snap_scalar(value.x, increments.x),
        snap_scalar(value.y, increments.y),
        snap_scalar(value.z, increments.z),
    )
}

pub(crate) fn apply_selection(
    state: &mut SelectionState,
    id: NodeId,
    mode: SelectionMode,
) -> SelectionChange {
    let previous = state.selected.clone();
    match mode {
        SelectionMode::Replace => {
            state.selected.clear();
            state.selected.push(id);
        }
        SelectionMode::Add => {
            if let Err(index) = state.selected.binary_search(&id) {
                state.selected.insert(index, id);
            }
        }
        SelectionMode::Toggle => match state.selected.binary_search(&id) {
            Ok(index) => {
                state.selected.remove(index);
            }
            Err(index) => state.selected.insert(index, id),
        },
        SelectionMode::Remove => {
            if let Ok(index) = state.selected.binary_search(&id) {
                state.selected.remove(index);
            }
        }
    }
    selection_diff(&previous, &state.selected)
}

pub(crate) fn replace_selection(
    state: &mut SelectionState,
    mut selected: Vec<NodeId>,
) -> SelectionChange {
    selected.sort_unstable();
    selected.dedup();
    let previous = core::mem::replace(&mut state.selected, selected);
    selection_diff(&previous, &state.selected)
}

fn selection_diff(previous: &[NodeId], current: &[NodeId]) -> SelectionChange {
    SelectionChange {
        added: current
            .iter()
            .copied()
            .filter(|id| previous.binary_search(id).is_err())
            .collect(),
        removed: previous
            .iter()
            .copied()
            .filter(|id| current.binary_search(id).is_err())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_settings_quantize_relative_deltas() {
        let snap = SnapSettings {
            translation: Vec3::new(0.5, 1.0, 0.0),
            rotation_radians: 0.25,
            scale: Vec3::new(0.1, 0.0, 0.5),
        };
        assert_eq!(
            snap.snap_translation(Vec3::new(0.74, 1.6, 0.3)),
            Vec3::new(0.5, 2.0, 0.3)
        );
        assert_eq!(snap.snap_rotation(0.38), 0.5);
    }
}
