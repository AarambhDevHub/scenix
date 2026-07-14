use scenix_core::{Inspectable, InspectorId, InspectorItem, InspectorSnapshot};

use crate::{PbrMaterial, PhysicalMaterial, UnlitMaterial};

impl Inspectable for PbrMaterial {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(InspectorId(1), &self.name, "pbr_material")
                .field("albedo", self.albedo)
                .field("metallic", self.metallic)
                .field("roughness", self.roughness)
                .field("emissive", self.emissive)
                .field("double_sided", self.double_sided),
        );
    }
}

impl Inspectable for PhysicalMaterial {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(InspectorId(1), &self.base.name, "physical_material")
                .field("albedo", self.base.albedo)
                .field("metallic", self.base.metallic)
                .field("roughness", self.base.roughness)
                .field("clearcoat", self.clearcoat)
                .field("sheen", self.sheen)
                .field("transmission", self.transmission)
                .field("ior", self.ior)
                .field("iridescence", self.iridescence),
        );
    }
}

impl Inspectable for UnlitMaterial {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(InspectorId(1), &self.name, "unlit_material")
                .field("color", self.color)
                .field("double_sided", self.double_sided),
        );
    }
}
