use scenix_core::{Inspectable, InspectorId, InspectorItem, InspectorSnapshot};

use crate::{AmbientLight, DirectionalLight, PointLight, SpotLight};

impl Inspectable for AmbientLight {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(InspectorId(1), "Ambient Light", "light")
                .field("color", self.color)
                .field("intensity", self.intensity),
        );
    }
}

impl Inspectable for DirectionalLight {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(InspectorId(1), "Directional Light", "light")
                .field("direction", self.direction)
                .field("color", self.color)
                .field("intensity", self.intensity)
                .field("shadows", self.shadow.is_some()),
        );
    }
}

impl Inspectable for PointLight {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(InspectorId(1), "Point Light", "light")
                .field("color", self.color)
                .field("intensity", self.intensity)
                .field("range", self.range)
                .field("decay", self.decay)
                .field("shadows", self.shadow.is_some()),
        );
    }
}

impl Inspectable for SpotLight {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(InspectorId(1), "Spot Light", "light")
                .field("color", self.color)
                .field("intensity", self.intensity)
                .field("range", self.range)
                .field("angle", self.angle)
                .field("penumbra", self.penumbra)
                .field("shadows", self.shadow.is_some()),
        );
    }
}
