//! Headless editor shell over the v1.10 tooling: an owning session with
//! exact inverse-command undo/redo, coalesced gizmo drags, scene-tree rows,
//! the node inspector surface, and project metadata — all solved on CPU and
//! printed, so host faces can mirror the flow one-to-one.
//!
//! Run with: `cargo run -p scenix --features editor`

use scenix::{
    DockKind, EditorSession, GizmoAxis, GizmoMode, InspectorEdit, PanelId, PanelSpec, PanelStack,
    PbrField, Quat, SceneNode, SelectionMode, Vec3, apply_pbr_field, scene_rows, standard_panels,
};

fn main() {
    // Own the graph and the standard five-panel shell.
    let mut session = EditorSession::new("Tool Shell Demo");
    let root = session.graph_mut().add(SceneNode::group("Hero"));
    let _ = session
        .graph_mut()
        .add_child(root, SceneNode::empty("Foot"))
        .expect("attach");
    *session.panels_mut() = standard_panels_with_assets();

    // Selection and inspector edits route through the history.
    session.select(root, SelectionMode::Replace).unwrap();
    let item = session.inspect_selected().expect("rows for selection");
    println!("inspector: {} ({} rows)", item.label, item.fields.len());
    session
        .apply_inspector_edit(root, InspectorEdit::SetPosition(Vec3::new(2.0, 0.0, 1.0)))
        .unwrap();
    session
        .apply_inspector_edit(
            root,
            InspectorEdit::SetRotation(Quat::from_axis_angle(Vec3::Y, 0.7)),
        )
        .unwrap();

    // Gizmo drags merge into one undo step inside the tick window.
    session
        .begin_drag(root, GizmoMode::Translate, GizmoAxis::X)
        .unwrap();
    session.drag_gizmo(0.4).unwrap();
    session.drag_gizmo(0.9).unwrap();
    session.end_drag();
    println!(
        "after drag: pos {:?} | undo steps {}",
        session.graph().get(root).unwrap().transform.translation,
        session.undo_len(),
    );

    // Console-style commands are the same enum the hosts submit.
    println!("undo -> {:?}", session.undo().map(|label| label.to_owned()));
    println!("redo -> {:?}", session.redo().map(|label| label.to_owned()));

    // Scene-tree rows are what a host panel paints per frame.
    for row in scene_rows(session.graph()) {
        println!(
            "tree d{} {}{}",
            row.depth,
            " ".repeat(row.depth * 2),
            row.label
        );
    }

    // Material edits are optimistic (host-owned undo), still logged.
    let mut material = scenix::PbrMaterial::new();
    println!(
        "material edit -> {}",
        apply_pbr_field(&mut material, PbrField::Metallic(0.8))
    );

    println!(
        "project title {} | schema {}",
        session.project_metadata().title,
        session.project_metadata().schema_version,
    );
}

/// The standard shell plus an open asset browser, as a plain panel edit.
fn standard_panels_with_assets() -> PanelStack {
    let mut panels = standard_panels();
    panels.upsert(PanelSpec::docked(
        PanelId(2),
        String::from("Assets"),
        DockKind::Bottom,
        240.0,
    ));
    panels
}
