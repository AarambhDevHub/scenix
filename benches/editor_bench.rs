use std::hint::black_box;
use std::time::Instant;

use scenix_editor::{
    EditCommand, EditorSession, GizmoAxis, GizmoMode, InspectorEdit, PbrField, PhysicalField,
    apply_pbr_field, apply_physical_field, axis_pick, scene_rows,
};
use scenix_material::{PbrMaterial, PhysicalMaterial};
use scenix_math::{Quat, Ray3, Transform, Vec3};
use scenix_scene::{NodeEditorMetadata, SceneNode, SelectionMode};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn build_session(nodes: usize) -> EditorSession {
    let mut session = EditorSession::new("Bench");
    {
        let graph = session.graph_mut();
        let root = graph.add(SceneNode::group("Root"));
        for index in 0..nodes {
            let child = SceneNode::empty(format!("node-{index}")).transform(Transform::new(
                Vec3::new(index as f32, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ));
            let _ = graph.add_child(root, child).expect("bench graph attach");
        }
    }
    session
}

fn main() {
    // History: apply + inverse computation per command, then undo.
    let mut session = build_session(64);
    let node = session.graph().iter_depth_first().nth(1).unwrap();
    bench("edit_command_roundtrip", 10_000, || {
        let command = EditCommand::SetTransform {
            id: node,
            transform: Transform::new(Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY, Vec3::ONE),
        };
        black_box(session.do_command(command).is_ok());
        let _ = session.undo();
    });
    session.clear_history();

    // Scene panel rows over mid-sized and capped graphs.
    bench("scene_rows_64", 20_000, || {
        black_box(scene_rows(black_box(session.graph())));
    });
    let wide = build_session(512);
    bench("scene_rows_512_cap", 5_000, || {
        black_box(scene_rows(black_box(wide.graph())));
    });

    // Gizmo solving: pick plus snapped delta application.
    let ray = Ray3 {
        origin: Vec3::new(0.5, 3.0, 0.05),
        direction: Vec3::new(0.0, -1.0, 0.0),
    };
    bench("axis_pick", 200_000, || {
        black_box(axis_pick(black_box(ray), Vec3::ZERO));
    });
    let mut drag_session = build_session(1);
    let node = drag_session.graph().iter_depth_first().next().unwrap();
    drag_session
        .begin_drag(node, GizmoMode::Translate, GizmoAxis::X)
        .expect("bench drag");
    bench("drag_gizmo_apply", 50_000, || {
        black_box(drag_session.drag_gizmo(0.37).is_ok());
    });

    // Inspector: row building plus edit routing.
    let mut inspector_session = build_session(16);
    let node = inspector_session.graph().iter_depth_first().nth(1).unwrap();
    inspector_session
        .select(node, SelectionMode::Replace)
        .expect("select");
    let _ = inspector_session
        .graph_mut()
        .set_editor_metadata(node, NodeEditorMetadata::default());
    bench("inspect_selected", 50_000, || {
        black_box(inspector_session.inspect_selected());
    });
    bench("inspector_position_edit", 20_000, || {
        black_box(
            inspector_session
                .apply_inspector_edit(node, InspectorEdit::SetPosition(Vec3::ONE))
                .is_ok(),
        );
        let _ = inspector_session.undo();
    });
    black_box(inspector_session);

    // Panel solve through the session layout.
    bench("session_layout", 50_000, || {
        black_box(wide.layout(scenix_math::Vec2::new(1600.0, 900.0)));
    });

    // Material field writes.
    let mut pbr = PbrMaterial::new();
    let mut physical = PhysicalMaterial::new();
    bench("material_field_writes", 100_000, || {
        black_box(apply_pbr_field(&mut pbr, PbrField::Metallic(0.4)));
        black_box(apply_physical_field(
            &mut physical,
            PhysicalField::Ior(1.45),
        ));
    });

    // Timeline advance.
    #[cfg(feature = "timeline")]
    bench_timeline();
}

#[cfg(feature = "timeline")]
fn bench_timeline() {
    use scenix_editor::TimelinePreview;

    let mut timeline = TimelinePreview::new(8.0);
    timeline.toggle_play();
    bench("timeline_advance", 200_000, || {
        black_box(timeline.advance(1.0 / 60.0));
    });
}
