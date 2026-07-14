use scenix_core::{Inspectable, InspectorId, InspectorItem, InspectorSnapshot, InspectorValue};

use crate::Renderer;

impl Inspectable for Renderer {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        let diagnostics = self.diagnostics();
        let editor = self.editor_buffer_stats();
        snapshot.push(
            InspectorItem::new(InspectorId(1), "Renderer", "wgpu")
                .field("frame", diagnostics.frame_index)
                .field("meshes", diagnostics.meshes as u64)
                .field("materials", diagnostics.materials as u64)
                .field("textures", diagnostics.textures as u64)
                .field("lights", diagnostics.lights as u64)
                .field(
                    "geometry_memory",
                    InspectorValue::Bytes(diagnostics.geometry_memory_bytes),
                )
                .field(
                    "texture_memory",
                    InspectorValue::Bytes(diagnostics.texture_memory_bytes),
                )
                .field("editor_buffers", editor.allocated)
                .field("editor_requests", editor.pick_requests)
                .field("editor_memory", InspectorValue::Bytes(editor.memory_bytes)),
        );
    }
}
