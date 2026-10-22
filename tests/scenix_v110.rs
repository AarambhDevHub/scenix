//! scenix v1.10.0 "Editor And UI Tooling" — facade-level CPU coverage.
//!
//! Exercises the two new tooling crates through the facade without a GPU or
//! window: owning editor sessions with exact inverse-command undo/redo and
//! coalesced gizmo drags, axis picking, scene-tree rows, the selected-node
//! inspector surface, material field edits, plus the renderer-agnostic UI
//! layer — docked panel stacks with the space solver, rolling renderer
//! stats, tool widgets, and bounded overlay composition. Asset-browser rows
//! run under `loader`, timeline previews under `animato`, and project JSON
//! exchange under `serde`, mirroring how hosts combine these features.

use scenix::{
    BudgetPolicy, COALESCE_WINDOW_TICKS, DockKind, DockedLayout, EditCommand, EditorSession,
    FrameSample, GIZMO_COALESCE_KEY, GIZMO_PICK_THRESHOLD, GizmoAxis, GizmoMode, GizmoSettings,
    HISTORY_DEFAULT_CAP, InspectorEdit, MAX_OVERLAY_RECTS, OverlayFrame, PANEL_INSPECTOR,
    PANEL_SCENE_TREE, PROJECT_SCHEMA_VERSION, PanelId, PanelSpec, PanelStack, PbrField,
    RendererStats, SCENE_PANEL_ROW_LIMIT, SLIDER_DEFAULT_STEP, STATS_HISTORY_FRAMES, SceneNode,
    SelectionMode, TextureSlot, Transform, Vec2, Vec3, WidgetEvent, apply_axis_delta,
    apply_pbr_field, apply_physical_field, assign_texture, axis_pick, compose_panel_overlay,
    kind_short_name, panel_placement, scene_rows, standard_panels, widget_hit,
};

#[test]
fn version_consts_match_the_release() {
    assert_eq!(scenix::UI_VERSION, "1.10.0");
    assert_eq!(PROJECT_SCHEMA_VERSION, 1);
    assert_eq!(HISTORY_DEFAULT_CAP, 128);
    assert_eq!(COALESCE_WINDOW_TICKS, 30);
    assert_eq!(GIZMO_COALESCE_KEY, 1);
    assert_eq!(SCENE_PANEL_ROW_LIMIT, 256);
    assert_eq!(STATS_HISTORY_FRAMES, 120);
    assert_eq!(GIZMO_PICK_THRESHOLD, 0.08);
    assert_eq!(SLIDER_DEFAULT_STEP, 0.01);
}

fn demo_session() -> EditorSession {
    let mut session = EditorSession::new("Facade Demo");
    let graph = session.graph_mut();
    let root = graph.add(SceneNode::group("Root"));
    graph
        .add_child(root, SceneNode::empty("Child"))
        .expect("child attaches");
    session
}

#[test]
fn editor_session_commands_roundtrip_through_history() {
    let mut session = demo_session();
    let root = session.graph().find_by_name("Root").expect("root exists");

    let label = session
        .do_command(EditCommand::RenameNode {
            id: root,
            name: String::from("Renamed"),
        })
        .expect("command applies");
    assert_eq!(label, "rename_node");
    assert_eq!(session.undo_len(), 1);
    assert_eq!(session.graph().get(root).unwrap().name, "Renamed");

    session.undo().expect("undo");
    assert_eq!(session.graph().get(root).unwrap().name, "Root");
    session.redo().expect("redo");
    assert_eq!(session.graph().get(root).unwrap().name, "Renamed");
    assert!(!session.can_redo(), "fresh work clears redo");

    // Overflow trims the oldest entries, keeping the stack bounded.
    for index in 0..HISTORY_DEFAULT_CAP + 16 {
        session
            .do_command(EditCommand::SetLayer {
                id: root,
                layer: index as u32,
            })
            .expect("layer edit");
        session.next_frame();
    }
    assert_eq!(session.undo_len(), HISTORY_DEFAULT_CAP);
    assert!(session.can_undo());
}

#[test]
fn gizmo_pick_and_coalesced_drag() {
    // The pointer ray grazes the +X handle at half its length.
    let ray = scenix::Ray3 {
        origin: Vec3::new(0.5, 3.0, 0.05),
        direction: Vec3::new(0.0, -1.0, 0.0),
    };
    let (axis, t) = axis_pick(ray, Vec3::ZERO).expect("handle picked");
    assert_eq!(axis, GizmoAxis::X);
    assert!((t - 0.5).abs() < 1.0e-4);

    let mut session = demo_session();
    let root = session.graph().find_by_name("Root").unwrap();
    session
        .begin_drag(root, GizmoMode::Translate, GizmoAxis::X)
        .expect("drag begins");

    // Two updates inside the coalescing window stay one undo entry, each
    // re-solving from the drag start so the same amount replays equally.
    let first = session.drag_gizmo(0.7).expect("drag applies");
    let second = session.drag_gizmo(1.2).expect("drag applies");
    assert_eq!(first.translation, Vec3::new(0.5, 0.0, 0.0));
    assert_eq!(second.translation, Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(session.undo_len(), 1);
    session.end_drag();

    session.undo().expect("undoes the whole drag");
    assert_eq!(
        session.graph().get(root).unwrap().transform,
        Transform::IDENTITY
    );

    // Direct axis math agrees with the session path.
    let solved = apply_axis_delta(
        Transform::IDENTITY,
        GizmoMode::Scale,
        GizmoAxis::Z,
        0.05,
        &GizmoSettings::SNAPPED,
    );
    assert_eq!(solved.scale, Vec3::new(1.0, 1.0, 1.1));
}

#[test]
fn scene_rows_and_inspector_surface_match_the_graph() {
    let mut session = demo_session();
    let root = session.graph().find_by_name("Root").unwrap();
    let child = session.graph().find_by_name("Child").unwrap();

    let rows = scene_rows(session.graph());
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].depth, 0);
    assert_eq!(rows[0].label, "Root");
    assert_eq!(kind_short_name(&SceneNode::group("probe").kind), "group");

    session
        .select(child, SelectionMode::Toggle)
        .expect("toggle");
    assert!(scene_rows(session.graph()).iter().any(|row| row.selected));

    session
        .select(root, SelectionMode::Replace)
        .expect("switch");
    let item = session.inspect_selected().expect("inspector row set");
    assert_eq!(item.label, "Root");
    let label = session
        .apply_inspector_edit(root, InspectorEdit::SetPosition(Vec3::new(2.0, 0.0, 0.0)))
        .expect("edit routes");
    assert_eq!(label, "set_node_transform");
    session.undo().expect("undo via the same history");
    assert_eq!(
        session.graph().get(root).unwrap().transform.translation,
        Vec3::ZERO
    );
    let _ = PANEL_SCENE_TREE;
    let _ = PANEL_INSPECTOR;
}

#[test]
fn material_field_edits_clamp_and_assign() {
    let mut material = scenix::PbrMaterial::new();
    apply_pbr_field(&mut material, PbrField::Metallic(1.5));
    assert_eq!(material.metallic, 1.0, "unit clamp holds");
    apply_pbr_field(
        &mut material,
        PbrField::AlbedoColor(scenix::Color::rgb(1.0, 0.5, 0.25)),
    );
    assert_eq!(material.albedo.to_array(), [1.0, 0.5, 0.25, 1.0]);

    let texture = scenix::TextureId::new(11);
    assign_texture(&mut material, TextureSlot::Emissive, Some(texture));
    assert_eq!(material.emissive_texture, Some(texture));
    assign_texture(&mut material, TextureSlot::Emissive, None);
    assert_eq!(material.emissive_texture, None);

    let mut physical = scenix::PhysicalMaterial::new();
    apply_physical_field(&mut physical, scenix::PhysicalField::Ior(3.0));
    assert_eq!(physical.ior, scenix::MATERIAL_IOR_MAX);
    apply_physical_field(&mut physical, scenix::PhysicalField::Clearcoat(-1.0));
    assert_eq!(physical.clearcoat, 0.0);
}

#[cfg(feature = "serde")]
#[test]
fn project_metadata_json_roundtrip() {
    use scenix::{ProjectMetadata, SceneRoute};

    let mut project = ProjectMetadata::new("Facade Project");
    project.author = String::from("scenix");
    project.add_scene(SceneRoute::new("Main", "scenes/main.scn"));
    project.validate().expect("record is valid");

    let json = project.to_json().expect("serializes");
    let parsed = ProjectMetadata::from_json(&json).expect("round trips");
    assert_eq!(parsed, project);
    assert_eq!(parsed.created_with, scenix::EDITOR_VERSION);
}

#[cfg(feature = "loader")]
#[test]
fn asset_browser_rows_filter_and_cap() {
    use scenix::AssetLoadStatus;
    use scenix::{AssetBrowserModel, AssetStatusClass, BROWSER_ROW_LIMIT};

    let mut browser = AssetBrowserModel::new();
    browser.push_row("assets/models/moon.glb", &AssetLoadStatus::Loaded);
    browser.push_row(
        "assets/models/sun.glb",
        &AssetLoadStatus::Loading { progress: 0.4 },
    );
    browser.set_filter("MOON");
    assert_eq!(browser.visible().count(), 1, "case-insensitive filter");
    browser.set_filter("");
    browser.set_status_filter(Some(AssetStatusClass::Loaded));
    assert_eq!(browser.visible().count(), 1);

    let mut full = AssetBrowserModel::new();
    for index in 0..BROWSER_ROW_LIMIT + 8 {
        full.push_row(format!("assets/row/{index}.glb"), &AssetLoadStatus::Loaded);
    }
    assert_eq!(full.rows().len(), BROWSER_ROW_LIMIT);
}

#[cfg(feature = "animato")]
#[test]
fn timeline_preview_tracks_animato_solving() {
    use scenix::LoopMode;
    use scenix::{TIMELINE_RATE_MAX, TimelinePreview};

    let mut timeline = TimelinePreview::new(2.0);
    timeline.playing = true;
    timeline.set_rate(TIMELINE_RATE_MAX);
    timeline.advance(0.25);
    assert!((timeline.time() - 2.0).abs() < 1.0e-5);

    timeline.set_loop_mode(LoopMode::Once);
    timeline.scrub(1.0);
    let frame = timeline.advance(1.0);
    assert!(frame.finished);
}

#[test]
fn panel_stack_space_solver_shapes_layouts() {
    let mut stack = PanelStack::new();
    stack.upsert(PanelSpec::docked(
        PanelId(0),
        String::from("Scene Tree"),
        DockKind::Left,
        300.0,
    ));
    stack.upsert(PanelSpec::docked(
        PanelId(1),
        String::from("Inspector"),
        DockKind::Right,
        340.0,
    ));
    let layout @ DockedLayout { .. } = panel_placement(&stack, Vec2::new(1600.0, 900.0));
    assert!((layout.left - 300.0).abs() < 1.0e-4);
    assert!((layout.right - 340.0).abs() < 1.0e-4);
    assert_eq!(layout.bottom, 0.0);

    // Standard sessions agree with the raw solver.
    let session = demo_session();
    let session_layout = session.layout(Vec2::new(1600.0, 900.0));
    assert_eq!(session_layout.left, layout.left);
    assert_eq!(session_layout.right, layout.right);
    assert_eq!(standard_panels().len(), 5);
}

#[test]
fn overlay_composition_stays_bounded_and_deterministic() {
    let viewport = Vec2::new(1024.0, 768.0);
    let mut stack = PanelStack::new();
    stack.upsert(PanelSpec::docked(
        PanelId(0),
        String::from("Scene Tree"),
        DockKind::Left,
        300.0,
    ));
    let mut stats = RendererStats::new();
    stats.record_frame(FrameSample::sanitized(16.7, 900, 480_000, 64 * 1024));
    stats.record_frame(FrameSample::sanitized(31.0, 1400, 900_000, 96 * 1024));

    let frame: OverlayFrame = compose_panel_overlay(viewport, &stack, Some(&stats));
    assert!(!frame.rects().is_empty(), "panel backdrops present");
    assert!(frame.rects().len() <= MAX_OVERLAY_RECTS);

    let again = compose_panel_overlay(viewport, &stack, Some(&stats));
    assert_eq!(frame.rects().len(), again.rects().len());
    assert!(frame.texts().len() <= again.texts().len() + 1);
}

#[test]
fn widgets_focus_and_hits_behave_like_tools() {
    use scenix::{Button, FocusChain, Slider, SmallTextBuffer, Toggle, WidgetId};

    let mut toggle = Toggle::OFF;
    assert_eq!(toggle.flip(), WidgetEvent::Toggled(true));
    assert!(toggle.on);

    let slider = Slider::new(0.0, 10.0);
    assert_eq!(slider.snap(5.0), 5.0);

    let mut button = Button::default();
    assert_eq!(button.press(), Some(WidgetEvent::Pressed));
    assert_eq!(button.press(), None, "held press is not a new edge");
    button.release();
    assert_eq!(button.press(), Some(WidgetEvent::Pressed));

    let mut buffer = SmallTextBuffer::new();
    buffer.push_str("scene");
    buffer.pop();
    assert_eq!(buffer.text(), "scen");

    let mut chain = FocusChain::new();
    chain.register(0, WidgetId(1));
    chain.register(2, WidgetId(3));
    assert_eq!(chain.focused(), Some(WidgetId(1)));
    assert_eq!(chain.focus_next(), Some(WidgetId(3)));
    assert_eq!(chain.focus_next(), Some(WidgetId(1)), "focus wraps");

    let rgba = scenix::ColorSwatch::from_rgba([255, 136, 0, 51]).hex();
    assert_eq!(rgba, "#FF880033");

    assert!(widget_hit(
        Vec2::new(32.0, 10.0),
        Vec2::ZERO,
        Vec2::new(64.0, 48.0)
    ));
    assert!(!widget_hit(
        Vec2::new(64.0, 10.0),
        Vec2::ZERO,
        Vec2::new(64.0, 48.0)
    ));
}

#[test]
fn renderer_stats_roll_with_budget_warnings() {
    let mut stats = RendererStats::new();
    for _ in 0..STATS_HISTORY_FRAMES + 10 {
        stats.record_frame(FrameSample::sanitized(33.4, 800, 300_000, 32 * 1024));
    }
    assert_eq!(stats.len(), STATS_HISTORY_FRAMES, "window rolling");

    let mut found = Vec::new();
    stats.budget_warnings(&BudgetPolicy::SIXTY_HZ, &mut found);
    assert!(
        !found.is_empty(),
        "30 ms frames trip the 60 Hz budget policy"
    );
}

#[test]
fn safe_rect_reexports_match_viewports() {
    // The facade carries the safe-area helpers overlay hosts need.
    let insets = scenix::Edges {
        left: 40.0,
        right: 40.0,
        top: 40.0,
        bottom: 40.0,
    };
    let rect = scenix::safe_rect(Vec2::new(800.0, 600.0), insets);
    assert_eq!(rect.min, Vec2::new(40.0, 40.0));
    assert_eq!(rect.size, Vec2::new(720.0, 520.0));
    assert_eq!(scenix::safe_rect(Vec2::ONE, insets).size, Vec2::ZERO);
}
