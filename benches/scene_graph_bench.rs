use std::hint::black_box;
use std::time::Instant;

use scenix_math::{Transform, Vec3};
use scenix_scene::{SceneGraph, SceneNode};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn build_graph(count: usize) -> SceneGraph {
    let mut graph = SceneGraph::with_capacity(count);
    let root = graph.add(SceneNode::group("root"));
    let mut parent = root;
    for index in 1..count {
        let node = SceneNode::new(format!("node-{index}"))
            .transform(Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)));
        let id = graph.add_child(parent, node).unwrap();
        if index % 8 == 0 {
            parent = root;
        } else {
            parent = id;
        }
    }
    graph
}

fn main() {
    let mut graph = build_graph(10_000);

    bench("scene_graph_update_world_transforms_10k", 100, || {
        let root = graph.roots()[0];
        graph
            .set_local_transform(
                root,
                Transform::from_translation(Vec3::new(black_box(1.0), 0.0, 0.0)),
            )
            .unwrap();
        graph.update_world_transforms();
    });

    bench("scene_graph_depth_first_10k", 100, || {
        let count = graph.iter_depth_first().count();
        black_box(count);
    });
}
