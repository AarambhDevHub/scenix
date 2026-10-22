use scenix_core::{InspectorId, InspectorItem, InspectorSnapshot, NodeId, ValidationError};
use scenix_math::{Quat, Vec3};

use crate::history::EditCommand;
use crate::panels::kind_short_name;
use crate::shell::EditorSession;

/// Hard cap on rows one selected-node item exposes before truncation.
pub const INSPECTOR_ROW_LIMIT: usize = 32;

/// Editor intents routed from inspector rows into session edit commands.
///
/// Mapping stays shallow on purpose: presence/boolean/layer edits map to
/// dedicated commands, transform field edits read the current transform and
/// only replace the touched lane, so partial inspector writes compose with
/// other edits instead of clobbering whole transforms.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InspectorEdit {
    /// Rename node.
    Rename(String),
    /// Toggle visibility.
    SetVisible(bool),
    /// Replace the culling layer bitmask.
    SetLayer(u32),
    /// Replace the translation lane.
    SetPosition(Vec3),
    /// Replace the rotation lane (host-supplied quaternion).
    SetRotation(Quat),
    /// Replace the scale lane.
    SetScale(Vec3),
}

impl EditorSession {
    /// Flat inspector rows for the interactively selected node.
    ///
    /// Falls back to the first selected id when no node is actively
    /// interacting (`SelectionState::active`); returns `None` with an empty
    /// selection, which hosts render as an empty panel.
    pub fn inspect_selected(&self) -> Option<InspectorItem> {
        let selection = self.graph().selection();
        let id = selection
            .active
            .or_else(|| selection.selected().first().copied())?;
        self.inspect_node(id)
    }

    /// Flat inspector rows for one node — the same vocabulary the v1.5
    /// snapshot tree uses but presented per-node for panel use.
    ///
    /// Transform lanes emit as scalar rows (position/rotation/scale per
    /// component) so hosts map rows to widget kinds without per-type
    /// parsing. Locked metadata still shows (greyable rows are a host
    /// concern); mutation is refused in [`EditorSession::apply_inspector_edit`].
    pub fn inspect_node(&self, id: NodeId) -> Option<InspectorItem> {
        let node = self.graph().get(id)?;
        let metadata = self.graph().editor_metadata(id);
        let label = metadata
            .and_then(|meta| meta.label.clone())
            .unwrap_or_else(|| node.name.clone());
        let transform = node.transform;

        let mut item = InspectorItem::new(InspectorId(0), label, kind_short_name(&node.kind))
            .field("visible", node.visible)
            .field("layer", node.layer as u64)
            .field("position", transform.translation)
            .field("rotation_x", transform.rotation.x)
            .field("rotation_y", transform.rotation.y)
            .field("rotation_z", transform.rotation.z)
            .field("rotation_w", transform.rotation.w)
            .field("scale", transform.scale);

        if let Some(meta) = metadata {
            item = item
                .field("locked", meta.locked)
                .field("tag_count", meta.tags.len());
        }
        Some(item)
    }

    /// Whole-graph inspector snapshot from the shipped v1.5 implementation.
    pub fn scene_snapshot(&self) -> InspectorSnapshot {
        use scenix_core::Inspectable;
        self.graph().inspector_snapshot()
    }

    /// Route an inspector edit through the session history.
    ///
    /// Locked nodes refuse the edit with [`ValidationError::InvalidState`];
    /// the row stays but the edit never lands, so a host can flash a
    /// "locked" affordance deterministically.
    pub fn apply_inspector_edit(
        &mut self,
        id: NodeId,
        edit: InspectorEdit,
    ) -> Result<&'static str, ValidationError> {
        let locked = self
            .graph()
            .editor_metadata(id)
            .map(|meta| meta.locked)
            .unwrap_or(false);
        if locked {
            return Err(ValidationError::InvalidState);
        }

        let command = match edit {
            InspectorEdit::Rename(name) => EditCommand::RenameNode { id, name },
            InspectorEdit::SetVisible(visible) => EditCommand::SetVisibility { id, visible },
            InspectorEdit::SetLayer(layer) => EditCommand::SetLayer { id, layer },
            InspectorEdit::SetPosition(position) => {
                let mut transform = self
                    .graph()
                    .get(id)
                    .ok_or(ValidationError::InvalidId)?
                    .transform;
                transform.translation = position;
                EditCommand::SetTransform { id, transform }
            }
            InspectorEdit::SetRotation(rotation) => {
                let mut transform = self
                    .graph()
                    .get(id)
                    .ok_or(ValidationError::InvalidId)?
                    .transform;
                transform.rotation = rotation;
                EditCommand::SetTransform { id, transform }
            }
            InspectorEdit::SetScale(scale) => {
                let mut transform = self
                    .graph()
                    .get(id)
                    .ok_or(ValidationError::InvalidId)?
                    .transform;
                transform.scale = scale;
                EditCommand::SetTransform { id, transform }
            }
        };
        self.do_command(command)
    }
}
