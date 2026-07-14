use scenix_core::{Inspectable, InspectorId, InspectorItem, InspectorSnapshot, InspectorValue};

use crate::{Texture2D, Texture3D, TextureCube};

impl Inspectable for Texture2D {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(
                InspectorId(1),
                self.label.as_deref().unwrap_or("Texture 2D"),
                "texture_2d",
            )
            .field("width", self.width as u64)
            .field("height", self.height as u64)
            .field("mip_levels", self.mip_levels as u64)
            .field("bytes", InspectorValue::Bytes(self.data.len() as u64)),
        );
    }
}

impl Inspectable for TextureCube {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        let bytes = self.faces.iter().map(|face| face.len()).sum::<usize>();
        snapshot.push(
            InspectorItem::new(
                InspectorId(1),
                self.label.as_deref().unwrap_or("Texture Cube"),
                "texture_cube",
            )
            .field("size", self.size as u64)
            .field("mip_levels", self.mip_levels as u64)
            .field("bytes", InspectorValue::Bytes(bytes as u64)),
        );
    }
}

impl Inspectable for Texture3D {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(
                InspectorId(1),
                self.label.as_deref().unwrap_or("Texture 3D"),
                "texture_3d",
            )
            .field("width", self.width as u64)
            .field("height", self.height as u64)
            .field("depth", self.depth as u64)
            .field("mip_levels", self.mip_levels as u64)
            .field("bytes", InspectorValue::Bytes(self.data.len() as u64)),
        );
    }
}
