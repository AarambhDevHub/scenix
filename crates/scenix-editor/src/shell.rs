use std::string::String;

use scenix_core::{NodeId, ValidationError};
use scenix_math::{Ray3, Transform, Vec2};
use scenix_scene::{SceneGraph, SelectionChange, SelectionMode};
use scenix_ui::{DockKind, DockedLayout, PanelId, PanelSpec, PanelStack, panel_placement};

use crate::gizmo::{GizmoAxis, GizmoMode, GizmoSettings, apply_axis_delta, axis_pick};
use crate::history::{EditCommand, EditHistory};
use crate::project::ProjectMetadata;

/// Scene tree panel id in the standard editor shells.
pub const PANEL_SCENE_TREE: PanelId = PanelId(0);
/// Inspector panel id.
pub const PANEL_INSPECTOR: PanelId = PanelId(1);
/// Asset browser panel id.
pub const PANEL_ASSETS: PanelId = PanelId(2);
/// Timeline panel id.
pub const PANEL_TIMELINE: PanelId = PanelId(3);
/// Material editor panel id.
pub const PANEL_MATERIAL: PanelId = PanelId(4);

/// Coalescing key every gizmo drag step shares, so a continuous drag is one
/// undo step while the tick window holds.
pub const GIZMO_COALESCE_KEY: u8 = 1;

/// The standard panel layout sessions start with: scene tree and inspector
/// docked open, asset browser, timeline, and material editor registered but
/// closed until a host opens them.
pub fn standard_panels() -> PanelStack {
    let mut panels = PanelStack::new();
    panels.upsert(PanelSpec::docked(
        PANEL_SCENE_TREE,
        String::from("Scene Tree"),
        DockKind::Left,
        300.0,
    ));
    panels.upsert(PanelSpec::docked(
        PANEL_INSPECTOR,
        String::from("Inspector"),
        DockKind::Right,
        340.0,
    ));
    panels.upsert(PanelSpec::docked(
        PANEL_ASSETS,
        String::from("Assets"),
        DockKind::Bottom,
        200.0,
    ));
    panels.upsert(PanelSpec::docked(
        PANEL_TIMELINE,
        String::from("Timeline"),
        DockKind::Bottom,
        220.0,
    ));
    panels.upsert(PanelSpec::docked(
        PANEL_MATERIAL,
        String::from("Material"),
        DockKind::Right,
        320.0,
    ));
    panels.close(PANEL_ASSETS);
    panels.close(PANEL_TIMELINE);
    panels.close(PANEL_MATERIAL);
    panels
}

/// One active gizmo drag, solved absolutely from `start` so re-ticking the
/// same amount always produces the same result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActiveGizmo {
    /// Node being manipulated.
    pub node: NodeId,
    /// Active transform tool.
    pub mode: GizmoMode,
    /// Active handle.
    pub axis: GizmoAxis,
    /// Transform captured when the drag began.
    pub start: Transform,
}

/// An owned scene graph plus its editor state: undo history, panel stack,
/// gizmo drag state, and session tick.
///
/// The session is the model half of an editor face — hosts call
/// [`EditorSession::next_frame`] once per frame, route pointer input through
/// the gizmo and selection methods, and read panels/layout for painting. All
/// mutations of graph content go through [`EditCommand`]s (concrete, inverse
/// computing edits) so undo and redo are exact.
pub struct EditorSession {
    graph: SceneGraph,
    history: EditHistory,
    panels: PanelStack,
    title: String,
    tick: u64,
    gizmo_settings: GizmoSettings,
    gizmo: Option<ActiveGizmo>,
}

impl EditorSession {
    /// New session named `title` with an empty graph and the standard panels.
    pub fn new(title: impl Into<String>) -> Self {
        Self::with_graph(title, SceneGraph::new())
    }

    /// New session adopting `graph` (history starts empty).
    pub fn with_graph(title: impl Into<String>, graph: SceneGraph) -> Self {
        Self {
            graph,
            history: EditHistory::new(),
            panels: standard_panels(),
            title: title.into(),
            tick: 0,
            gizmo_settings: GizmoSettings::SNAPPED,
            gizmo: None,
        }
    }

    /// Session title (window header / project default).
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Read-only scene graph access.
    pub fn graph(&self) -> &SceneGraph {
        &self.graph
    }

    /// Mutable graph access for hosts doing out-of-band edits; such edits
    /// bypass the history and are the host's undo obligation.
    pub fn graph_mut(&mut self) -> &mut SceneGraph {
        &mut self.graph
    }

    /// Panel stack (visibility, docks, thicknesses).
    pub fn panels(&self) -> &PanelStack {
        &self.panels
    }

    /// Mutable panel stack for host UI actions.
    pub fn panels_mut(&mut self) -> &mut PanelStack {
        &mut self.panels
    }

    /// Current gizmo snapping settings.
    pub fn gizmo_settings(&self) -> GizmoSettings {
        self.gizmo_settings
    }

    /// Replace gizmo snapping settings.
    pub fn set_gizmo_settings(&mut self, settings: GizmoSettings) {
        self.gizmo_settings = settings;
    }

    /// Session tick — hosts call once per frame; the coalescing window and
    /// history timestamps use it.
    pub fn next_frame(&mut self) {
        self.tick += 1;
    }

    /// Current tick value.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Solve the docked panel layout for a viewport size.
    pub fn layout(&self, viewport: Vec2) -> DockedLayout {
        panel_placement(&self.panels, viewport)
    }

    /// Apply one edit command through the history.
    ///
    /// Returns the stable action label; the inverse was computed from the
    /// pre-state and queued for undo.
    pub fn do_command(&mut self, command: EditCommand) -> Result<&'static str, ValidationError> {
        let label = command.label();
        let inverse = command.clone().apply(&mut self.graph)?;
        self.history.record(command, inverse, None, self.tick);
        Ok(label)
    }

    /// Apply a transform update coalesced under [`GIZMO_COALESCE_KEY`], so
    /// gizmo drags merge into one undo step while the tick window holds.
    pub fn do_transform(
        &mut self,
        id: NodeId,
        transform: Transform,
    ) -> Result<&'static str, ValidationError> {
        let label = EditCommand::SetTransform { id, transform }.label();
        let command = EditCommand::SetTransform { id, transform };
        let inverse = command.clone().apply(&mut self.graph)?;
        self.history
            .record(command, inverse, Some(GIZMO_COALESCE_KEY), self.tick);
        Ok(label)
    }

    /// Undo the newest edit, returning its label.
    ///
    /// Fails cleanly (returns `None`) when the inverse no longer applies —
    /// for example after the host deleted the node out-of-band — leaving the
    /// remaining history untouched.
    pub fn undo(&mut self) -> Option<&'static str> {
        let entry = self.history.pop_undo()?;
        let label = entry.label;
        let forward = entry.applied;
        match entry.inverse.apply(&mut self.graph) {
            Ok(_) => {
                self.history.push_redo(forward);
                Some(label)
            }
            // The recorded world moved on; drop the step rather than corrupt.
            Err(_) => None,
        }
    }

    /// Redo the newest reverted edit, returning its label.
    pub fn redo(&mut self) -> Option<&'static str> {
        let command = self.history.pop_redo()?;
        let label = command.label();
        let inverse = command.clone().apply(&mut self.graph).ok()?;
        self.history.push_undo_entry(command, inverse);
        Some(label)
    }

    /// Whether at least one undo step exists.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Whether at least one redo step exists.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Retained undo steps.
    pub fn undo_len(&self) -> usize {
        self.history.undo_len()
    }

    /// Queued redo steps.
    pub fn redo_len(&self) -> usize {
        self.history.redo_len()
    }

    /// Label of the next undo step, for menu rows.
    pub fn peek_undo_label(&self) -> Option<&'static str> {
        self.history.peek_undo_label()
    }

    /// Wipe the history without touching graph content.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Forward a selection request into the graph.
    pub fn select(
        &mut self,
        id: NodeId,
        mode: SelectionMode,
    ) -> Result<SelectionChange, ValidationError> {
        self.graph.select(id, mode)
    }

    /// Clear the graph selection.
    pub fn clear_selection(&mut self) -> SelectionChange {
        self.graph.clear_selection()
    }

    /// Begin a gizmo drag on `node`.
    ///
    /// Rejects when the node is unknown ([`ValidationError::InvalidId`]),
    /// not transformable or already dragged ([`ValidationError::InvalidState`]).
    pub fn begin_drag(
        &mut self,
        node: NodeId,
        mode: GizmoMode,
        axis: GizmoAxis,
    ) -> Result<(), ValidationError> {
        if self.gizmo.is_some() {
            return Err(ValidationError::InvalidState);
        }
        let start_transform = self
            .graph
            .get(node)
            .ok_or(ValidationError::InvalidId)?
            .transform;
        if !self.graph.is_transformable(node) {
            return Err(ValidationError::InvalidState);
        }
        self.gizmo = Some(ActiveGizmo {
            node,
            mode,
            axis,
            start: start_transform,
        });
        Ok(())
    }

    /// Report one raw drag amount (world units for translate/scale, radians
    /// for rotate) against the active gizmo.
    ///
    /// The delta re-solves from the drag start each call, so replaying the
    /// same amount reproduces the exact transform; each update enters the
    /// history coalesced into one gesture entry.
    pub fn drag_gizmo(&mut self, amount: f32) -> Result<Transform, ValidationError> {
        let (node, mode, axis, start) = match self.gizmo {
            Some(active) => (active.node, active.mode, active.axis, active.start),
            None => return Err(ValidationError::InvalidState),
        };
        let transform = apply_axis_delta(start, mode, axis, amount, &self.gizmo_settings);
        self.do_transform(node, transform)?;
        Ok(transform)
    }

    /// End the active drag, returning its descriptor.
    pub fn end_drag(&mut self) -> Option<ActiveGizmo> {
        self.gizmo.take()
    }

    /// The active gizmo drag, if any.
    pub fn active_drag(&self) -> Option<&ActiveGizmo> {
        self.gizmo.as_ref()
    }

    /// Pick a gizmo axis handle from a pointer ray against `node`'s world
    /// position. Call `graph.update_world_transforms()` beforehand so the
    /// cached world transform is current.
    pub fn pick_gizmo(&self, ray: Ray3, node: NodeId) -> Option<(GizmoAxis, f32)> {
        let origin = self.graph.world_transform(node)?.translation;
        axis_pick(ray, origin)
    }

    /// Snapshot a fresh project metadata record for this session.
    pub fn project_metadata(&self) -> ProjectMetadata {
        ProjectMetadata::from_session(self)
    }
}
