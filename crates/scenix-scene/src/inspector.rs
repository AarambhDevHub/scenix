use alloc::string::String;

use scenix_core::{Inspectable, InspectorId, InspectorItem, InspectorSnapshot};

use crate::{NodeKind, SceneGraph};

impl Inspectable for SceneGraph {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        let mut root = InspectorItem::new(InspectorId(0), "Scene", "scene_graph")
            .field("nodes", self.iter_depth_first().count())
            .field("selected", self.selection().selected().len());
        for id in self.roots() {
            if let Some(item) = inspect_node(self, *id) {
                root.children.push(item);
            }
        }
        snapshot.push(root);
    }
}

fn inspect_node(scene: &SceneGraph, id: scenix_core::NodeId) -> Option<InspectorItem> {
    let node = scene.get(id)?;
    let metadata = scene.editor_metadata(id).cloned().unwrap_or_default();
    if !metadata.visible_in_inspector {
        return None;
    }
    let label = metadata.label.unwrap_or_else(|| node.name.clone());
    let kind = match node.kind {
        NodeKind::Empty => "empty",
        NodeKind::Group => "group",
        NodeKind::Mesh { .. } => "mesh",
        NodeKind::Light { .. } => "light",
        NodeKind::Camera { .. } => "camera",
        NodeKind::Sprite(_) => "sprite",
    };
    let mut item = InspectorItem::new(InspectorId(id.get()), label, kind)
        .field("id", id.get())
        .field("visible", node.visible)
        .field(
            "layers",
            String::from(if node.layer == u32::MAX {
                "all"
            } else {
                "custom"
            }),
        )
        .field("translation", node.transform.translation)
        .field("scale", node.transform.scale)
        .field("selected", scene.selection().contains(id))
        .field("locked", metadata.locked);
    if let Some(children) = scene.children(id) {
        for child in children {
            if let Some(child) = inspect_node(scene, *child) {
                item.children.push(child);
            }
        }
    }
    Some(item)
}
