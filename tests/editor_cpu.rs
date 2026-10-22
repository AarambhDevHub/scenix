//! Distributed `scenix-editor` v1.10.0 CPU validation suite.
//!
//! Run from the repository root:
//!
//! ```sh
//! cargo test -p scenix-editor --test editor_cpu
//! ```

use scenix_core::{Color, InspectorValue, MaterialId, TextureId, ValidationError};
use scenix_editor::{
    EDITOR_VERSION, EditCommand, EditorSession, GizmoAxis, GizmoMode, GizmoSettings,
    HISTORY_DEFAULT_CAP, InspectorEdit, MaterialEditorLog, PROJECT_SCHEMA_VERSION, PbrField,
    PhysicalField, ROTATE_SNAP_RADIANS, SCENE_PANEL_ROW_LIMIT, TextureSlot, apply_pbr_field,
    apply_physical_field, assign_texture, axis_pick, scene_rows, standard_panels,
};
use scenix_math::{Quat, Ray3, Transform, Vec3};
use scenix_scene::{NodeEditorMetadata, SceneGraph, SceneNode, SelectionMode};

fn demo_session() -> EditorSession {
    let mut session = EditorSession::new("Editor Demo");
    let graph = session.graph_mut();
    let root = graph.add(SceneNode::group("Root"));
    let child = graph
        .add_child(root, SceneNode::empty("Child"))
        .expect("child attaches");
    let _ = graph
        .add_child(child, SceneNode::empty("Leaf"))
        .expect("leaf attaches");
    session
}

#[test]
fn editor_version_matches_workspace_release() {
    assert_eq!(EDITOR_VERSION, "1.10.0");
    assert_eq!(PROJECT_SCHEMA_VERSION, 1);
}

#[test]
fn edit_commands_apply_and_invert() {
    let mut graph = SceneGraph::new();
    let root = graph.add(SceneNode::group("Root"));
    let child = graph
        .add_child(root, SceneNode::empty("Child"))
        .expect("child attaches");

    // Rename round trip.
    let inverse = EditCommand::RenameNode {
        id: child,
        name: String::from("Renamed"),
    }
    .apply(&mut graph)
    .expect("rename applies");
    assert_eq!(graph.get(child).expect("node").name, "Renamed");
    inverse.apply(&mut graph).expect("inverse applies");
    assert_eq!(graph.get(child).expect("node").name, "Child");

    // Visibility round trip.
    let inverse = EditCommand::SetVisibility {
        id: child,
        visible: false,
    }
    .apply(&mut graph)
    .expect("visibility applies");
    assert!(!graph.get(child).expect("node").visible);
    inverse.apply(&mut graph).expect("inverse applies");
    assert!(graph.get(child).expect("node").visible);

    // Layer round trip (nodes start fully visible on every bit of the mask).
    let start_layer = graph.get(child).expect("node").layer;
    let inverse = EditCommand::SetLayer {
        id: child,
        layer: 7,
    }
    .apply(&mut graph)
    .expect("layer applies");
    assert_eq!(graph.get(child).expect("node").layer, 7);
    inverse.apply(&mut graph).expect("inverse applies");
    assert_eq!(graph.get(child).expect("node").layer, start_layer);

    // Transform round trip.
    let target = Transform::new(Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY, Vec3::ONE);
    let inverse = EditCommand::SetTransform {
        id: child,
        transform: target,
    }
    .apply(&mut graph)
    .expect("transform applies");
    assert_eq!(graph.get(child).expect("node").transform, target);
    inverse.apply(&mut graph).expect("inverse applies");
    assert_eq!(
        graph.get(child).expect("node").transform,
        Transform::IDENTITY
    );

    // Reparent round trip (no cycles).
    let other = graph.add(SceneNode::group("Other"));
    let inverse = EditCommand::ReparentNode {
        id: child,
        parent: Some(other),
    }
    .apply(&mut graph)
    .expect("reparent applies");
    assert_eq!(graph.parent(child), Some(other));
    inverse.apply(&mut graph).expect("inverse applies");
    assert_eq!(graph.parent(child), Some(root));
}

#[test]
fn session_commands_drive_history() {
    let mut session = EditorSession::new("History Demo");
    let root = session.graph_mut().add(SceneNode::group("Root"));

    let label = session
        .do_command(EditCommand::RenameNode {
            id: root,
            name: String::from("Renamed"),
        })
        .expect("rename accepted");
    assert_eq!(label, "rename_node");
    assert_eq!(session.undo_len(), 1);
    assert_eq!(session.peek_undo_label(), Some("rename_node"));

    let undone = session.undo().expect("undo works");
    assert_eq!(undone, "rename_node");
    assert_eq!(session.graph().get(root).expect("node").name, "Root");
    assert_eq!(session.redo_len(), 1);

    let redone = session.redo().expect("redo works");
    assert_eq!(redone, "rename_node");
    assert_eq!(session.graph().get(root).expect("node").name, "Renamed");
    assert!(!session.can_redo(), "new work clears redo");

    // Overflow keeps the stack bounded, dropping the oldest entries.
    for index in 0..(HISTORY_DEFAULT_CAP + 32) {
        session
            .do_command(EditCommand::SetLayer {
                id: root,
                layer: index as u32,
            })
            .expect("layer edit accepted");
        session.next_frame();
    }
    assert_eq!(session.undo_len(), HISTORY_DEFAULT_CAP);
    session.clear_history();
    assert!(!session.can_undo());
}

#[test]
fn gizmo_drag_coalesces_into_one_undo_step() {
    let mut session = demo_session();
    let root = session.graph().find_by_name("Root").expect("root exists");

    session
        .begin_drag(root, GizmoMode::Translate, GizmoAxis::X)
        .expect("drag begins");
    for amount in [0.7, 1.2] {
        session.drag_gizmo(amount).expect("drag applies");
    }
    assert_eq!(
        session.undo_len(),
        1,
        "continuous same-key drags merge into one entry"
    );
    assert_eq!(
        session.graph().get(root).unwrap().transform.translation,
        Vec3::new(1.0, 0.0, 0.0)
    );

    session.end_drag();
    session.undo().expect("undo of the whole drag");
    assert_eq!(
        session.graph().get(root).unwrap().transform,
        Transform::IDENTITY,
        "undo restores the pre-drag pose"
    );

    // A second drag begins cleanly; a nested one is refused.
    session
        .begin_drag(root, GizmoMode::Rotate, GizmoAxis::Z)
        .expect("second drag begins");
    assert!(
        session
            .begin_drag(root, GizmoMode::Scale, GizmoAxis::Y)
            .is_err()
    );
}

#[test]
fn axis_pick_picks_nearest_handle() {
    // Pointer ray coming straight down, offset slightly off the +X handle.
    let ray = Ray3 {
        origin: Vec3::new(0.5, 3.0, 0.05),
        direction: Vec3::new(0.0, -1.0, 0.0),
    };
    let (axis, t) = axis_pick(ray, Vec3::ZERO).expect("X handle picked");
    assert_eq!(axis, GizmoAxis::X);
    assert!((t - 0.5).abs() < 1.0e-4, "handle param {t}");

    // A ray missing every handle finds nothing.
    let miss = Ray3 {
        origin: Vec3::new(5.0, 3.0, 5.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
    };
    assert!(axis_pick(miss, Vec3::ZERO).is_none());
}

#[test]
fn axis_delta_snapping_and_freeform() {
    let mut session = demo_session();
    let root = session.graph().find_by_name("Root").unwrap();

    // Translation snaps to half units.
    session
        .begin_drag(root, GizmoMode::Translate, GizmoAxis::X)
        .unwrap();
    let moved = session.drag_gizmo(0.7).unwrap();
    assert_eq!(moved.translation, Vec3::new(0.5, 0.0, 0.0));
    session.end_drag();

    // Rotation snaps to 15 degree steps about the world axis.
    session
        .begin_drag(root, GizmoMode::Rotate, GizmoAxis::Z)
        .unwrap();
    let rotated = session.drag_gizmo(0.3).unwrap();
    let expected = Quat::from_axis_angle(Vec3::Z, ROTATE_SNAP_RADIANS);
    assert!((rotated.rotation.z - expected.z).abs() < 1.0e-6);
    assert!((rotated.rotation.w - expected.w).abs() < 1.0e-6);
    session.end_drag();

    // Scale snaps to tenths per handle.
    session
        .begin_drag(root, GizmoMode::Scale, GizmoAxis::Y)
        .unwrap();
    let scaled = session.drag_gizmo(0.05).unwrap();
    assert_eq!(scaled.scale, Vec3::new(1.0, 1.1, 1.0));
    session.end_drag();

    // Free settings never snap.
    session.set_gizmo_settings(GizmoSettings::FREE);
    session
        .begin_drag(root, GizmoMode::Translate, GizmoAxis::Z)
        .unwrap();
    let free = session.drag_gizmo(0.3).unwrap();
    assert_eq!(free.translation, Vec3::new(0.5, 0.0, 0.3));
}

fn row_value<'a>(item: &'a scenix_core::InspectorItem, name: &str) -> Option<&'a InspectorValue> {
    item.fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| &field.value)
}

#[cfg(feature = "inspector")]
#[test]
fn inspector_rows_and_edit_routing() {
    let mut session = demo_session();
    let child = session.graph().find_by_name("Child").unwrap();
    session
        .select(child, SelectionMode::Replace)
        .expect("selection made");

    let item = session.inspect_selected().expect("a selection exists");
    assert_eq!(item.label, "Child");
    assert_eq!(item.kind, "empty");
    assert!(matches!(
        row_value(&item, "visible"),
        Some(InspectorValue::Bool(true))
    ));
    assert!(matches!(
        row_value(&item, "position"),
        Some(InspectorValue::Vec3(_))
    ));

    // Edit routing flips the data through the history.
    let label = session
        .apply_inspector_edit(child, InspectorEdit::SetVisible(false))
        .expect("edit routes");
    assert_eq!(label, "set_node_visibility");
    assert!(!session.graph().get(child).unwrap().visible);
    assert_eq!(session.undo_len(), 1);
    session.undo().expect("undo");
    assert!(session.graph().get(child).unwrap().visible);

    // Position lane edits compose with the untouched rotation lane.
    let label = session
        .apply_inspector_edit(child, InspectorEdit::SetPosition(Vec3::new(4.0, 5.0, 6.0)))
        .expect("position edit routes");
    assert_eq!(label, "set_node_transform");
    let node = session.graph().get(child).unwrap();
    assert_eq!(node.transform.translation, Vec3::new(4.0, 5.0, 6.0));
    assert_eq!(node.transform.scale, Vec3::ONE);
    let _ = session.scene_snapshot();

    // Locked nodes refuse edits deterministically.
    session
        .graph_mut()
        .select(child, SelectionMode::Replace)
        .expect("select for lock test");
    let mut metadata = NodeEditorMetadata::default();
    metadata.locked = true;
    session
        .graph_mut()
        .set_editor_metadata(child, metadata)
        .expect("metadata sets");
    assert_eq!(
        session.apply_inspector_edit(child, InspectorEdit::SetLayer(2)),
        Err(ValidationError::InvalidState)
    );
}

#[test]
fn scene_rows_flatten_and_select() {
    let mut session = demo_session();
    let root = session.graph().find_by_name("Root").unwrap();
    let child = session.graph().find_by_name("Child").unwrap();

    let rows = scene_rows(session.graph());
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].id, root, "roots walk first");
    assert_eq!(rows[0].depth, 0);
    assert_eq!(rows[0].label, "Root");
    assert_eq!(rows[0].kind, "group");
    assert_eq!(rows[1].depth, 1);
    assert!(rows.iter().all(|row| !row.selected));

    session
        .select(child, SelectionMode::Toggle)
        .expect("toggle");
    assert!(
        scene_rows(session.graph())
            .iter()
            .any(|row| row.id == child && row.selected)
    );

    session.clear_selection();
    assert!(scene_rows(session.graph()).iter().all(|row| !row.selected));
}

#[test]
fn scene_rows_respect_the_cap() {
    let mut session = EditorSession::new("Wide Scene");
    {
        let graph = session.graph_mut();
        for index in 0..SCENE_PANEL_ROW_LIMIT + 20 {
            graph.add(SceneNode::empty(format!("node-{index}")));
        }
    }
    assert_eq!(scene_rows(session.graph()).len(), SCENE_PANEL_ROW_LIMIT);
}

#[cfg(feature = "material")]
#[test]
fn material_field_writes_clamp_and_log() {
    use scenix_editor::{MATERIAL_IOR_MAX, MATERIAL_UNIT_MAX};

    let mut material = scenix_material::PbrMaterial::new();
    assert_eq!(
        apply_pbr_field(&mut material, PbrField::Metallic(1.7)),
        "set_material_metallic"
    );
    assert_eq!(material.metallic, MATERIAL_UNIT_MAX);
    assert_eq!(
        apply_pbr_field(&mut material, PbrField::Roughness(-0.4)),
        "set_material_roughness"
    );
    assert_eq!(material.roughness, 0.0);

    apply_pbr_field(
        &mut material,
        PbrField::AlbedoColor(Color::rgb(1.0, 0.5, 0.25)),
    );
    assert_eq!(material.albedo.to_array(), [1.0, 0.5, 0.25, 1.0]);

    let texture = TextureId::new(7);
    assert_eq!(
        assign_texture(&mut material, TextureSlot::Normal, Some(texture)),
        "assign_pbr_normal_texture"
    );
    assert_eq!(material.normal_texture, Some(texture));
    assign_texture(&mut material, TextureSlot::Normal, None);
    assert_eq!(material.normal_texture, None);

    let mut physical = scenix_material::PhysicalMaterial::new();
    assert_eq!(
        apply_physical_field(&mut physical, PhysicalField::Ior(3.0)),
        "set_physical_ior"
    );
    assert_eq!(physical.ior, MATERIAL_IOR_MAX);
    assert_eq!(
        apply_physical_field(&mut physical, PhysicalField::Clearcoat(-1.0)),
        "set_physical_clearcoat"
    );
    assert_eq!(physical.clearcoat, 0.0);

    let mut log = MaterialEditorLog::new();
    let material_id = MaterialId::new(3);
    log.record(material_id, "set_material_metallic");
    log.record(material_id, "set_material_roughness");
    log.record(MaterialId::new(9), "set_material_name");
    assert_eq!(log.len(), 3);
    assert_eq!(log.changes_for(material_id), 2);
    log.clear();
    assert!(log.is_empty());
}

#[cfg(feature = "assets")]
#[test]
fn browser_rows_filter_and_cap() {
    use scenix_editor::{
        AssetBrowserModel, AssetBrowserRow, AssetStatusClass, BROWSER_LABEL_CAP, BROWSER_ROW_LIMIT,
    };
    use scenix_loader::AssetLoadStatus;

    let mut browser = AssetBrowserModel::new();
    browser.push_row("assets/models/moon.glb", &AssetLoadStatus::Loaded);
    browser.push_row(
        "assets/models/sun.glb",
        &AssetLoadStatus::Loading { progress: 0.4 },
    );
    browser.push_row("assets/textures/moon-map.png", &AssetLoadStatus::Cancelled);
    browser.push_row("assets/audio/ambience.ogg", &AssetLoadStatus::Pending);

    assert_eq!(browser.visible().count(), 4);
    browser.set_filter("MOON");
    assert_eq!(browser.visible().count(), 2, "case-insensitive substring");
    browser.set_filter("");
    browser.set_status_filter(Some(AssetStatusClass::Cancelled));
    assert_eq!(browser.visible().count(), 1);
    assert_eq!(
        browser.visible().next().unwrap().label,
        "assets/textures/moon-map.png"
    );
    browser.set_status_filter(None);

    let loading = browser
        .rows()
        .iter()
        .find(|row| row.status == AssetStatusClass::Loading)
        .unwrap();
    assert_eq!(loading.progress, 0.4);
    assert_eq!(browser.count_by(AssetStatusClass::Loaded), 1);

    // Row cap and label truncation both hold.
    for index in 0..BROWSER_ROW_LIMIT + 8 {
        browser.push_row(format!("assets/row/{index}.glb"), &AssetLoadStatus::Loaded);
    }
    assert_eq!(browser.rows().len(), BROWSER_ROW_LIMIT);

    let long = "x".repeat(BROWSER_LABEL_CAP + 12);
    let row = AssetBrowserRow::new(long, &AssetLoadStatus::Loaded);
    assert_eq!(row.label.chars().count(), BROWSER_LABEL_CAP);
    assert_eq!(row.progress, 1.0);
}

#[cfg(feature = "timeline")]
#[test]
fn timeline_preview_playback_and_scrub() {
    use scenix_animato::LoopMode;
    use scenix_editor::{TIMELINE_RATE_MAX, TIMELINE_SNAP_FPS, TimelinePreview};

    let mut timeline = TimelinePreview::new(2.0);
    assert!(!timeline.playing);
    let idle = timeline.advance(0.5);
    assert!(!idle.finished);
    assert!((idle.time - 0.0).abs() < 1.0e-6);
    assert!(timeline.toggle_play());

    timeline.advance(0.5);
    assert!((timeline.time() - 0.5).abs() < 1.0e-6);

    timeline.set_rate(2.0);
    timeline.advance(0.25);
    assert!(
        (timeline.time() - 1.0).abs() < 1.0e-5,
        "rate doubles the delta"
    );
    timeline.set_rate(TIMELINE_RATE_MAX + 10.0);
    assert_eq!(timeline.rate, TIMELINE_RATE_MAX);

    // Scrubbing snaps to the 60 FPS grid.
    timeline.scrub(1.777);
    let expected = 107.0 / TIMELINE_SNAP_FPS;
    assert!(
        (timeline.time() - expected).abs() < 1.0e-4,
        "snapped {}",
        timeline.time()
    );

    // Once mode pins the playhead and reports finished.
    timeline.set_loop_mode(LoopMode::Once);
    timeline.set_duration(1.0);
    timeline.scrub(0.9);
    timeline.playing = true;
    let frame = timeline.advance(1.0);
    assert!(frame.finished);
    assert!((frame.time - 1.0).abs() < 1.0e-5);
    let again = timeline.advance(1.0);
    assert!(again.finished, "stays pinned, keeps reporting");

    // Finite repeats cap on max iterations.
    let mut capped = TimelinePreview::new(1.0);
    capped.set_loop_mode(LoopMode::Repeat { max: 2 });
    capped.toggle_play();
    let frame = capped.advance(5.0);
    assert!(frame.finished);
    assert_eq!(frame.iteration, 2);
}

#[test]
fn project_metadata_validates_and_roundtrips() {
    use scenix_editor::{ProjectError, ProjectMetadata, SceneRoute};

    let mut project = ProjectMetadata::new("Demo");
    project.author = String::from("Team");
    project.add_scene(SceneRoute::new("Main", "scenes/main.scn"));
    project.add_scene(SceneRoute::new("Menu", "scenes/menu.scn"));
    project.validate().expect("in-memory record is valid");
    assert_eq!(project.created_with, EDITOR_VERSION);

    let duplicate = ProjectMetadata {
        scenes: vec![SceneRoute::new("Main", "a"), SceneRoute::new("Main", "b")],
        ..ProjectMetadata::new("Dup")
    };
    assert_eq!(duplicate.validate(), Err(ProjectError::DuplicateScene));

    let empty = ProjectMetadata::new("   ");
    assert_eq!(empty.validate(), Err(ProjectError::EmptyTitle));

    let future = ProjectMetadata {
        schema_version: PROJECT_SCHEMA_VERSION + 1,
        ..ProjectMetadata::new("Future")
    };
    assert_eq!(future.validate(), Err(ProjectError::UnsupportedSchema));

    // Schema notes: older records move forward; future records refuse.
    let mut older = ProjectMetadata::new("Old");
    older.schema_version = 0;
    assert!(older.upgrade_to_current().expect("upgrade"));
    assert!(!older.upgrade_to_current().expect("already current"));

    #[cfg(feature = "serde")]
    {
        let json = project.to_json().expect("serializes");
        let parsed = ProjectMetadata::from_json(&json).expect("round trip");
        assert_eq!(parsed, project);
        assert!(json.contains("\"schema_version\": 1"));
    }
}

#[test]
fn session_layout_and_panels_agree() {
    let session = demo_session();
    let layout = session.layout(scenix_math::Vec2::new(1600.0, 900.0));
    assert!((layout.left - 300.0).abs() < 1.0e-4, "scene tree docked");
    assert!((layout.right - 340.0).abs() < 1.0e-4, "inspector docked");
    assert_eq!(layout.bottom, 0.0, "assets/timeline start closed");

    let panels = session.panels();
    assert!(panels.is_visible(scenix_editor::PANEL_SCENE_TREE));
    assert!(panels.is_visible(scenix_editor::PANEL_INSPECTOR));
    assert!(!panels.is_visible(scenix_editor::PANEL_ASSETS));
    assert_eq!(
        standard_panels().len(),
        5,
        "five standard panels registered"
    );

    // Fresh session histories begin empty.
    assert!(!session.can_undo());
    assert!(!session.can_redo());
    assert_eq!(session.tick(), 0);
    assert_eq!(session.title(), "Editor Demo");
}
