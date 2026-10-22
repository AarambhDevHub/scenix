use std::string::String;
use std::vec::Vec;

use scenix_core::NodeId;
use scenix_scene::{NodeKind, SceneGraph};

/// Upper bound on rows one scene-tree solve emits.
///
/// Huge scenes stream beyond this; the panel scrolls rather than growing the
/// frame cost without limit. The bound is part of the API contract, not a
/// tuned constant.
pub const SCENE_PANEL_ROW_LIMIT: usize = 256;

/// One flattened scene-tree row for a host panel.
///
/// Depth drives indentation; selection/hidden/locked let a host paint one
/// row style without touching graph state itself.
#[derive(Clone, Debug, PartialEq)]
pub struct ScenePanelRow {
    /// Node the row represents.
    pub id: NodeId,
    /// Hierarchy depth (roots at zero).
    pub depth: usize,
    /// Editor label: metadata label when present, else the node name.
    pub label: String,
    /// Short payload tag (`"mesh"`, `"light"`, …) for badge painting.
    pub kind: &'static str,
    /// Whether the row is inside the graph's selection set.
    pub selected: bool,
    /// Whether the node is hidden from rendering.
    pub hidden: bool,
    /// Whether editor mutations are locked on the node.
    pub locked: bool,
}

/// Short payload tag for a node kind, matching the inspector vocabulary.
pub fn kind_short_name(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Empty => "empty",
        NodeKind::Group => "group",
        NodeKind::Mesh { .. } => "mesh",
        NodeKind::Light { .. } => "light",
        NodeKind::Camera { .. } => "camera",
        NodeKind::Sprite(_) => "sprite",
        NodeKind::Particles { .. } => "particles",
        NodeKind::Terrain { .. } => "terrain",
        NodeKind::Sky { .. } => "sky",
        NodeKind::XrAnchor { .. } => "xr_anchor",
        NodeKind::AudioSource { .. } => "audio_source",
        NodeKind::PhysicsBody { .. } => "physics_body",
        NodeKind::Water { .. } => "water",
    }
}

/// Flatten the graph into scene-panel rows, depth first, roots in order.
///
/// Deterministic two-stack walk (children keep registration order, roots keep
/// root order); output is capped at [`SCENE_PANEL_ROW_LIMIT`]. Selection,
/// hidden, and locked flags are read at solve time, so re-running after a
/// session step refreshes every row without diffing.
pub fn scene_rows(graph: &SceneGraph) -> Vec<ScenePanelRow> {
    let selected: &[NodeId] = graph.selection().selected();
    let mut stack: Vec<(NodeId, usize)> = graph.roots().iter().rev().map(|id| (*id, 0)).collect();
    let mut rows: Vec<ScenePanelRow> = Vec::new();

    while let Some((id, depth)) = stack.pop() {
        let node = match graph.get(id) {
            Some(node) => node,
            None => continue,
        };
        let metadata = graph.editor_metadata(id).cloned();
        let label = metadata
            .as_ref()
            .and_then(|meta| meta.label.clone())
            .unwrap_or_else(|| node.name.clone());
        rows.push(ScenePanelRow {
            id,
            depth,
            label,
            kind: kind_short_name(&node.kind),
            selected: selected.contains(&id),
            hidden: !node.visible,
            locked: metadata.map(|meta| meta.locked).unwrap_or(false),
        });
        if rows.len() >= SCENE_PANEL_ROW_LIMIT {
            break;
        }
        if let Some(children) = graph.children(id) {
            for child in children.iter().rev() {
                stack.push((*child, depth + 1));
            }
        }
    }
    rows
}
