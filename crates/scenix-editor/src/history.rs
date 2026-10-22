use std::string::String;
use std::vec::Vec;

use scenix_core::{NodeId, ValidationError};
use scenix_math::Transform;
use scenix_scene::SceneGraph;

/// Default maximum number of undo steps retained by an [`EditHistory`].
pub const HISTORY_DEFAULT_CAP: usize = 128;

/// How many session ticks a gizmo-style coalescing key keeps merging for.
///
/// Continuous drags produce dozens of `SetTransform` updates per second;
/// merging them by key plus tick window keeps the undo stack at gesture
/// granularity instead of flooding it with one step per pointer move.
pub const COALESCE_WINDOW_TICKS: u64 = 30;

/// One concrete, invertible scene edit.
///
/// Commands are an enum rather than a trait: they stay `Clone` + `PartialEq`
/// (and serde-serializable under the `serde` feature) so sessions can log,
/// replay, and persist the exact edit sequence a designer performed.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EditCommand {
    /// Renames a node.
    RenameNode { id: NodeId, name: String },
    /// Shows or hides a node and its payload.
    SetVisibility { id: NodeId, visible: bool },
    /// Assigns the camera culling layer bitmask.
    SetLayer { id: NodeId, layer: u32 },
    /// Replaces the local transform (position, rotation, scale).
    SetTransform { id: NodeId, transform: Transform },
    /// Moves a node under a new parent (`None` reparents to a root).
    ReparentNode { id: NodeId, parent: Option<NodeId> },
}

impl EditCommand {
    /// Stable label for history rows and telemetry.
    pub fn label(&self) -> &'static str {
        match self {
            Self::RenameNode { .. } => "rename_node",
            Self::SetVisibility { .. } => "set_node_visibility",
            Self::SetLayer { .. } => "set_node_layer",
            Self::SetTransform { .. } => "set_node_transform",
            Self::ReparentNode { .. } => "reparent_node",
        }
    }

    /// Whether `self` and `other` may merge into one history entry.
    ///
    /// Only same-node transform updates coalesce: renames, visibility flips,
    /// and reparents are each log-worthy events on their own.
    pub fn coalesces_with(&self, other: &EditCommand) -> bool {
        matches!(
            (self, other),
            (Self::SetTransform { id: a, .. }, Self::SetTransform { id: b, .. }) if a == b
        )
    }

    /// Applies the command to `graph`, returning the inverse command that
    /// restores the prior state.
    ///
    /// The read happens before the write, so `undo` works even when the
    /// current value equals the command's target. Node-targeting failures
    /// surface as [`ValidationError::InvalidId`] before any mutation.
    pub fn apply(self, graph: &mut SceneGraph) -> Result<EditCommand, ValidationError> {
        match self {
            Self::RenameNode { id, name } => {
                let node = graph.get_mut(id).ok_or(ValidationError::InvalidId)?;
                let inverse = Self::RenameNode {
                    id,
                    name: node.name.clone(),
                };
                node.name = name;
                Ok(inverse)
            }
            Self::SetVisibility { id, visible } => {
                let node = graph.get_mut(id).ok_or(ValidationError::InvalidId)?;
                let inverse = Self::SetVisibility {
                    id,
                    visible: node.visible,
                };
                node.visible = visible;
                Ok(inverse)
            }
            Self::SetLayer { id, layer } => {
                let node = graph.get_mut(id).ok_or(ValidationError::InvalidId)?;
                let inverse = Self::SetLayer {
                    id,
                    layer: node.layer,
                };
                node.layer = layer;
                Ok(inverse)
            }
            Self::SetTransform { id, transform } => {
                let inverse = Self::SetTransform {
                    id,
                    transform: graph.get(id).ok_or(ValidationError::InvalidId)?.transform,
                };
                graph.set_local_transform(id, transform)?;
                Ok(inverse)
            }
            Self::ReparentNode { id, parent } => {
                let inverse = Self::ReparentNode {
                    id,
                    parent: graph.parent(id),
                };
                graph.reparent(id, parent)?;
                Ok(inverse)
            }
        }
    }
}

/// One undo entry: what was applied, what restores the prior state, and how
/// continuous edits grouped.
#[derive(Clone, Debug, PartialEq)]
pub struct HistoryEntry {
    /// Stable label of the applied command.
    pub label: &'static str,
    /// The forward command (re-applied by redo).
    pub applied: EditCommand,
    /// The inverse command (applied by undo).
    pub inverse: EditCommand,
    /// Coalescing key when the entry belongs to a continuous gesture.
    pub coalesce: Option<u8>,
    /// Session tick when the entry was recorded.
    pub tick: u64,
}

/// Bounded undo/redo stack of scene edits.
///
/// Overflow drops the oldest entry first (`remove(0)` on the rare overflow
/// only), redo clears on every new command, and continuous gizmo-style input
/// merges under a key + tick window so one drag is one undo step.
#[derive(Clone, Debug, Default)]
pub struct EditHistory {
    undo: Vec<HistoryEntry>,
    redo: Vec<EditCommand>,
    cap: usize,
}

impl EditHistory {
    /// History with [`HISTORY_DEFAULT_CAP`] undo steps.
    pub fn new() -> Self {
        Self::with_capacity(HISTORY_DEFAULT_CAP)
    }

    /// History with a custom undo cap.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            cap,
        }
    }

    /// Record one applied edit with its inverse.
    ///
    /// `coalesce` groups continuous commands (same key, same node, ticks
    /// within [`COALESCE_WINDOW_TICKS`]) into the existing entry, keeping the
    /// *first* inverse so undo restores the pre-gesture scene.
    pub fn record(
        &mut self,
        applied: EditCommand,
        inverse: EditCommand,
        coalesce: Option<u8>,
        tick: u64,
    ) {
        let label = applied.label();
        if let (Some(key), Some(top)) = (coalesce, self.undo.last_mut())
            && top.coalesce == Some(key)
            && tick.saturating_sub(top.tick) <= COALESCE_WINDOW_TICKS
            && top.applied.coalesces_with(&applied)
        {
            top.applied = applied;
            top.tick = tick;
            self.redo.clear();
            return;
        }
        self.undo.push(HistoryEntry {
            label,
            applied,
            inverse,
            coalesce,
            tick,
        });
        if self.undo.len() > self.cap {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Pops the newest undo entry, handing the session both directions.
    pub fn pop_undo(&mut self) -> Option<HistoryEntry> {
        self.undo.pop()
    }

    /// Queues a forward command for redo.
    pub fn push_redo(&mut self, command: EditCommand) {
        self.redo.push(command);
    }

    /// Pops the newest redo command.
    pub fn pop_redo(&mut self) -> Option<EditCommand> {
        self.redo.pop()
    }

    /// Push an undo entry without touching the redo stack.
    ///
    /// Exclusively for redo replay: the redo command was popped, applied, and
    /// its freshly computed inverse records here while remaining redo steps
    /// stay queued.
    pub fn push_undo_entry(&mut self, applied: EditCommand, inverse: EditCommand) {
        let label = applied.label();
        self.undo.push(HistoryEntry {
            label,
            applied,
            inverse,
            coalesce: None,
            tick: 0,
        });
        if self.undo.len() > self.cap {
            self.undo.remove(0);
        }
    }

    /// Undo entries currently retained.
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    /// Redo commands currently queued.
    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    /// Whether at least one undo step exists.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether at least one redo step exists.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Label of the next undo step, for menu rows.
    pub fn peek_undo_label(&self) -> Option<&'static str> {
        self.undo.last().map(|entry| entry.label)
    }

    /// Wipes both directions (host swapped the scene graph).
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}
