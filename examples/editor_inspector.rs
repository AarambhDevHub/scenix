use scenix::{Inspectable, InspectorSnapshot, SceneGraph, SceneNode, show_inspector};

fn main() {
    let mut scene = SceneGraph::new();
    scene.add(SceneNode::group("Editor Root"));
    let snapshot: InspectorSnapshot = scene.inspector_snapshot();

    let context = egui::Context::default();
    let output = context.run(Default::default(), |context| {
        egui::CentralPanel::default().show(context, |ui| {
            ui.heading("scenix inspector");
            let _ = show_inspector(ui, &snapshot);
        });
    });
    println!("inspector produced {} paint shapes", output.shapes.len());
}
