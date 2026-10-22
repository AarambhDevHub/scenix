use std::string::String;
use std::vec::Vec;

use scenix_core::{Color, MaterialId, TextureId};
use scenix_material::{PbrMaterial, PhysicalMaterial};
use scenix_math::Vec3;

/// Clamp band for unit material fields (metallic, roughness, sheen, …).
pub const MATERIAL_UNIT_MAX: f32 = 1.0;

/// Clamp band for index-of-refraction values ([`MATERIAL_IOR_MIN`], max).
pub const MATERIAL_IOR_MIN: f32 = 1.0;
/// Upper IOR clamp.
pub const MATERIAL_IOR_MAX: f32 = 2.5;

/// Upper bound of retained material change entries.
pub const MATERIAL_LOG_DEFAULT_CAP: usize = 128;

/// Writable PBR base fields, values pre-clamped by [`apply_pbr_field`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PbrField {
    /// Display name.
    Name(String),
    /// Base color in linear RGBA.
    AlbedoColor(Color),
    /// Metalness in `0..=1`.
    Metallic(f32),
    /// Roughness in `0..=1`.
    Roughness(f32),
    /// Emissive RGB in linear space.
    Emissive(Vec3),
    /// Two-sided rendering flag.
    DoubleSided(bool),
}

/// Assignable texture slots on [`PbrMaterial`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextureSlot {
    /// Base color texture.
    Albedo,
    /// Packed metallic-roughness texture.
    MetallicRoughness,
    /// Tangent-space normal map.
    Normal,
    /// Ambient occlusion texture.
    Occlusion,
    /// Emissive mask/color texture.
    Emissive,
}

/// Assign a texture (`None` clears the slot), returning the change label.
pub fn assign_texture(
    material: &mut PbrMaterial,
    slot: TextureSlot,
    texture: Option<TextureId>,
) -> &'static str {
    match slot {
        TextureSlot::Albedo => {
            material.albedo_texture = texture;
            "assign_pbr_albedo_texture"
        }
        TextureSlot::MetallicRoughness => {
            material.metallic_roughness_texture = texture;
            "assign_pbr_metallic_roughness_texture"
        }
        TextureSlot::Normal => {
            material.normal_texture = texture;
            "assign_pbr_normal_texture"
        }
        TextureSlot::Occlusion => {
            material.occlusion_texture = texture;
            "assign_pbr_occlusion_texture"
        }
        TextureSlot::Emissive => {
            material.emissive_texture = texture;
            "assign_pbr_emissive_texture"
        }
    }
}

/// Writable physical-extension fields ([`PhysicalMaterial`]).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PhysicalField {
    /// Clearcoat layer strength in `0..=1`.
    Clearcoat(f32),
    /// Fabric sheen strength in `0..=1`.
    Sheen(f32),
    /// Glass transmission in `0..=1`.
    Transmission(f32),
    /// Index of refraction in `[1.0, 2.5]`.
    Ior(f32),
    /// Thin-film iridescence in `0..=1`.
    Iridescence(f32),
}

/// Write one PBR field with editor clamps, returning the change label.
pub fn apply_pbr_field(material: &mut PbrMaterial, field: PbrField) -> &'static str {
    match field {
        PbrField::Name(name) => {
            material.name = name;
            "set_material_name"
        }
        PbrField::AlbedoColor(color) => {
            material.albedo = color;
            "set_material_albedo"
        }
        PbrField::Metallic(value) => {
            material.metallic = value.clamp(0.0, MATERIAL_UNIT_MAX);
            "set_material_metallic"
        }
        PbrField::Roughness(value) => {
            material.roughness = value.clamp(0.0, MATERIAL_UNIT_MAX);
            "set_material_roughness"
        }
        PbrField::Emissive(emissive) => {
            material.emissive = emissive;
            "set_material_emissive"
        }
        PbrField::DoubleSided(double_sided) => {
            material.double_sided = double_sided;
            "set_material_double_sided"
        }
    }
}

/// Write one physical-extension field with editor clamps.
pub fn apply_physical_field(material: &mut PhysicalMaterial, field: PhysicalField) -> &'static str {
    match field {
        PhysicalField::Clearcoat(value) => {
            material.clearcoat = value.clamp(0.0, MATERIAL_UNIT_MAX);
            "set_physical_clearcoat"
        }
        PhysicalField::Sheen(value) => {
            material.sheen = value.clamp(0.0, MATERIAL_UNIT_MAX);
            "set_physical_sheen"
        }
        PhysicalField::Transmission(value) => {
            material.transmission = value.clamp(0.0, MATERIAL_UNIT_MAX);
            "set_physical_transmission"
        }
        PhysicalField::Ior(value) => {
            material.ior = value.clamp(MATERIAL_IOR_MIN, MATERIAL_IOR_MAX);
            "set_physical_ior"
        }
        PhysicalField::Iridescence(value) => {
            material.iridescence = value.clamp(0.0, MATERIAL_UNIT_MAX);
            "set_physical_iridescence"
        }
    }
}

/// One material change row: owning material plus the action label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaterialChange {
    /// Material the edit touched.
    pub material: MaterialId,
    /// Stable action label from the apply functions.
    pub label: &'static str,
}

/// Bounded change log for material editing.
///
/// Materials live inside the host's resource store, so — unlike graph edits —
/// undo wiring is the host's job; the log exists so host surfaces can show
/// and replay what the material panel changed per material.
#[derive(Clone, Debug, Default)]
pub struct MaterialEditorLog {
    changes: Vec<MaterialChange>,
    cap: usize,
}

impl MaterialEditorLog {
    /// Log with [`MATERIAL_LOG_DEFAULT_CAP`] retention.
    pub fn new() -> Self {
        Self::with_capacity(MATERIAL_LOG_DEFAULT_CAP)
    }

    /// Log with a custom retention.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            changes: Vec::new(),
            cap,
        }
    }

    /// Record one change (oldest entries drop past the cap).
    pub fn record(&mut self, material: MaterialId, label: &'static str) {
        self.changes.push(MaterialChange { material, label });
        if self.changes.len() > self.cap {
            self.changes.remove(0);
        }
    }

    /// Recorded changes in application order.
    pub fn changes(&self) -> &[MaterialChange] {
        &self.changes
    }

    /// Number of recorded changes.
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Whether no change is recorded.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Number of changes touching `material`.
    pub fn changes_for(&self, material: MaterialId) -> usize {
        self.changes
            .iter()
            .filter(|change| change.material == material)
            .count()
    }

    /// Wipe the log (save checkpoint).
    pub fn clear(&mut self) {
        self.changes.clear();
    }
}
