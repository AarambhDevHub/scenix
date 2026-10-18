//! `Inspectable` snapshots for node artifacts behind the `inspector` feature.
//!
//! Node materials and post effects appear in the scene inspector exactly
//! like built-in materials: one row per artifact with read-only typed
//! fields the egui/WASM frontends can render.

use scenix_core::inspector::{Inspectable, InspectorId, InspectorItem, InspectorSnapshot};

use crate::material::NodeMaterial;
use crate::post::PostNodeEffect;

/// Incrementing snapshot-local identifier source.
struct IdSource(u64);

impl IdSource {
    fn next(&mut self) -> InspectorId {
        self.0 += 1;
        InspectorId(self.0)
    }
}

impl Inspectable for NodeMaterial {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        let mut ids = IdSource(0);
        let mut item = InspectorItem::new(ids.next(), self.name(), "NodeMaterial")
            .field("nodes", self.graph().len())
            .field("base_color", self.base_color())
            .field("emissive", self.emissive())
            .field("metallic", self.metallic())
            .field("roughness", self.roughness())
            .field("double_sided", self.double_sided())
            .field("uses_texture", self.uses_texture());
        if let Some(cutoff) = self.alpha_cutoff() {
            item = item.field("alpha_cutoff", cutoff);
        }
        snapshot.push(item);
    }
}

impl Inspectable for PostNodeEffect {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        let mut ids = IdSource(0);
        let item = InspectorItem::new(ids.next(), self.name(), "PostNodeEffect")
            .field("nodes", self.graph().len());
        snapshot.push(item);
    }
}
