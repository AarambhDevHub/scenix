use scenix_core::{Color, MeshId, NodeId, TextureId, ValidationError};
use scenix_math::{Mat4, Transform, Vec3};
use scenix_scene::{BillboardMode, Fog, LodGroup, SceneGraph, SceneNode, Sprite};

fn assert_close(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-4, "{a} != {b}");
}

fn world_translation(graph: &SceneGraph, id: NodeId) -> Vec3 {
    graph.world_matrix(id).unwrap().mul_vec3(Vec3::ZERO)
}

#[test]
fn root_and_child_insertion_preserve_hierarchy_invariants() {
    let mut graph = SceneGraph::new();
    let root = graph.add(SceneNode::group("root"));
    let child = graph.add_child(root, SceneNode::new("child")).unwrap();

    assert_eq!(graph.roots(), &[root]);
    assert_eq!(graph.parent(root), None);
    assert_eq!(graph.parent(child), Some(root));
    assert_eq!(graph.children(root).unwrap(), &[child]);
    assert_eq!(graph.children(child).unwrap(), &[]);
}

#[test]
fn invalid_ids_return_validation_errors() {
    let mut graph = SceneGraph::new();
    let missing = NodeId::new(99);

    assert_eq!(
        graph.add_child(missing, SceneNode::new("child")),
        Err(ValidationError::InvalidId)
    );
    assert_eq!(graph.remove(missing), Err(ValidationError::InvalidId));
    assert_eq!(
        graph.reparent(missing, None),
        Err(ValidationError::InvalidId)
    );
}

#[test]
fn reparent_updates_roots_and_rejects_cycles() {
    let mut graph = SceneGraph::new();
    let a = graph.add(SceneNode::group("a"));
    let b = graph.add(SceneNode::group("b"));
    let child = graph.add_child(a, SceneNode::new("child")).unwrap();
    let grandchild = graph
        .add_child(child, SceneNode::new("grandchild"))
        .unwrap();

    assert_eq!(
        graph.reparent(a, Some(grandchild)),
        Err(ValidationError::InvalidState)
    );

    graph.reparent(child, Some(b)).unwrap();
    assert_eq!(graph.parent(child), Some(b));
    assert_eq!(graph.children(a).unwrap(), &[]);
    assert_eq!(graph.children(b).unwrap(), &[child]);

    graph.reparent(child, None).unwrap();
    assert_eq!(graph.parent(child), None);
    assert!(graph.roots().contains(&child));
}

#[test]
fn remove_cascades_iteratively_through_deep_trees() {
    let mut graph = SceneGraph::new();
    let root = graph.add(SceneNode::group("root"));
    let mut parent = root;
    let mut ids = vec![root];
    for index in 0..1024 {
        let child = graph
            .add_child(parent, SceneNode::new(format!("child-{index}")))
            .unwrap();
        ids.push(child);
        parent = child;
    }

    graph.remove(root).unwrap();

    assert!(graph.roots().is_empty());
    for id in ids {
        assert!(graph.get(id).is_none());
        assert!(graph.world_matrix(id).is_none());
    }
}

#[test]
fn world_transforms_compose_across_deep_chains() {
    let mut graph = SceneGraph::new();
    let mut parent = graph.add(
        SceneNode::new("root").transform(Transform::from_translation(Vec3::new(1.0, 0.0, 0.0))),
    );
    let mut last = parent;

    for index in 0..9 {
        last = graph
            .add_child(
                parent,
                SceneNode::new(format!("node-{index}"))
                    .transform(Transform::from_translation(Vec3::new(1.0, 0.0, 0.0))),
            )
            .unwrap();
        parent = last;
    }

    graph.update_world_transforms();
    let translation = world_translation(&graph, last);
    assert_close(translation.x, 10.0);
    assert_close(translation.y, 0.0);
}

#[test]
fn dirty_propagation_updates_subtrees_after_mutation() {
    let mut graph = SceneGraph::new();
    let root = graph.add(
        SceneNode::new("root").transform(Transform::from_translation(Vec3::new(1.0, 0.0, 0.0))),
    );
    let child = graph
        .add_child(
            root,
            SceneNode::new("child")
                .transform(Transform::from_translation(Vec3::new(2.0, 0.0, 0.0))),
        )
        .unwrap();

    graph.update_world_transforms();
    assert_close(world_translation(&graph, child).x, 3.0);

    graph.get_mut(root).unwrap().transform = Transform::from_translation(Vec3::new(3.0, 0.0, 0.0));
    graph.update_world_transforms();
    assert_close(world_translation(&graph, child).x, 5.0);

    graph
        .set_local_transform(child, Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)))
        .unwrap();
    graph.update_world_transforms();
    assert_close(world_translation(&graph, child).x, 7.0);
}

#[test]
fn traversal_order_is_deterministic() {
    let mut graph = SceneGraph::new();
    let a = graph.add(SceneNode::group("a"));
    let e = graph.add(SceneNode::group("e"));
    let b = graph.add_child(a, SceneNode::new("b")).unwrap();
    let c = graph.add_child(a, SceneNode::new("c")).unwrap();
    let d = graph.add_child(b, SceneNode::new("d")).unwrap();

    let depth: Vec<_> = graph.iter_depth_first().collect();
    let breadth: Vec<_> = graph.iter_breadth_first().collect();

    assert_eq!(depth, vec![a, b, d, c, e]);
    assert_eq!(breadth, vec![a, e, b, c, d]);
}

#[test]
fn find_by_name_uses_depth_first_order() {
    let mut graph = SceneGraph::new();
    let root = graph.add(SceneNode::group("root"));
    let first = graph.add_child(root, SceneNode::new("target")).unwrap();
    let second = graph.add(SceneNode::new("target"));

    assert_eq!(graph.find_by_name("target"), Some(first));
    assert_ne!(graph.find_by_name("target"), Some(second));
    assert_eq!(graph.find_by_name("missing"), None);
}

#[test]
fn scene_support_types_work() {
    let fog = Fog::linear(0.5, 100.0, Color::WHITE);
    let sprite = Sprite::new(2.0, 3.0, TextureId::new(9)).billboard(BillboardMode::FaceCamera);

    let mut graph = SceneGraph::new();
    graph.set_fog(Some(fog));
    let sprite_id = graph.add(SceneNode::sprite("sprite", sprite));

    assert_eq!(graph.fog(), Some(fog));
    assert!(graph.get(sprite_id).unwrap().visible);

    let lod = LodGroup::new(vec![
        (100.0, MeshId::new(3)),
        (5.0, MeshId::new(1)),
        (20.0, MeshId::new(2)),
    ]);
    assert_eq!(lod.select(0.0), Some(MeshId::new(1)));
    assert_eq!(lod.select(5.0), Some(MeshId::new(1)));
    assert_eq!(lod.select(6.0), Some(MeshId::new(2)));
    assert_eq!(lod.select(999.0), Some(MeshId::new(3)));
}

#[test]
fn invalid_world_queries_return_none() {
    let graph = SceneGraph::new();
    assert_eq!(graph.world_matrix(NodeId::new(1)), None);
    assert_eq!(graph.world_transform(NodeId::new(1)), None);
    assert_eq!(graph.world_matrix(NodeId::default()), None);
    assert_eq!(Mat4::IDENTITY.to_cols_array()[0], 1.0);
}
