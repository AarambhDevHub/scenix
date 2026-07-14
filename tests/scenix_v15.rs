use std::collections::BTreeMap;

use scenix::{
    ArcballController, DragController, DragPlane, GizmoGeometry, InputState, MaterialId, MeshId,
    NodeEditorMetadata, PerspectiveCamera, Ray3, Raycaster, SceneGraph, SceneNode,
    SelectionFrustum, SelectionMode, SelectionRect, SnapSettings, TransformGizmoHelper,
    TransformMode, Vec2, Vec3, box_geometry,
};

#[test]
fn v15_controls_selection_drag_and_gizmo_work_together() {
    let mut input = InputState::default();
    input.on_scroll(-0.5);
    let mut control = ArcballController::default();
    control.update_from_input(&input, 1.0 / 60.0);
    let mut camera = PerspectiveCamera::default();
    control.apply_to_perspective(&mut camera);

    let mesh_id = MeshId::new(1);
    let mut geometries = BTreeMap::new();
    geometries.insert(mesh_id, box_geometry(1.0, 1.0, 1.0, 1, 1, 1));
    let mut scene = SceneGraph::new();
    let id = scene.add(SceneNode::mesh("cube", mesh_id, MaterialId::new(1)));
    scene.update_world_transforms();

    let mut raycaster = Raycaster::new();
    raycaster.build_bvh(&scene, &geometries).unwrap();
    let selected = raycaster.select_in_frustum(
        SelectionFrustum::from_perspective(
            &camera,
            SelectionRect::from_ndc(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0)),
        ),
        &scene,
        &geometries,
    );
    assert_eq!(selected, vec![id]);
    scene.select(id, SelectionMode::Replace).unwrap();

    let plane = DragPlane::from_normal(Vec3::ZERO, Vec3::Z);
    let mut drag = DragController::default();
    drag.begin(
        &mut scene,
        id,
        Ray3::new(Vec3::Z, Vec3::NEG_Z),
        plane,
        SnapSettings::default(),
    )
    .unwrap();
    drag.update(&mut scene, Ray3::new(Vec3::new(1.0, 0.0, 1.0), Vec3::NEG_Z))
        .unwrap();
    drag.end(&mut scene).unwrap();
    assert_eq!(scene.get(id).unwrap().transform.translation.x, 1.0);

    let mut gizmo = GizmoGeometry::default();
    TransformGizmoHelper::new(Vec3::X, TransformMode::Translate).write_geometry(&mut gizmo);
    assert_eq!(gizmo.handles.len(), 7);
}

#[test]
fn locked_metadata_blocks_editor_mutation() {
    let mut scene = SceneGraph::new();
    let id = scene.add(SceneNode::new("locked"));
    scene
        .set_editor_metadata(
            id,
            NodeEditorMetadata {
                locked: true,
                ..NodeEditorMetadata::default()
            },
        )
        .unwrap();
    assert!(scene.select(id, SelectionMode::Replace).is_err());
    assert!(!scene.is_transformable(id));
}

#[test]
fn selection_is_sorted_and_removal_clears_editor_state() {
    let mut scene = SceneGraph::new();
    let parent = scene.add(SceneNode::group("parent"));
    let child = scene.add_child(parent, SceneNode::new("child")).unwrap();
    scene.select_many([child, parent, child]).unwrap();
    scene.set_hovered(Some(child)).unwrap();
    scene.set_active(Some(child)).unwrap();

    assert_eq!(scene.selection().selected(), &[parent, child]);
    scene.remove(parent).unwrap();
    assert!(scene.selection().selected().is_empty());
    assert_eq!(scene.selection().hovered, None);
    assert_eq!(scene.selection().active, None);
}
