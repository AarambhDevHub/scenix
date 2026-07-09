//! Typed property bindings for clip-based animation.
//!
//! Scenix keeps a typed-ID discipline: instead of stringly property paths like
//! Three.js's `"position.x"`, bindings are `(id, typed_property)` pairs. The
//! facade `clip_from_loaded` helper maps glTF node indices to scene `NodeId`s
//! and produces these bindings.

use scenix_core::{CameraId, LightId, MaterialId, MeshId, NodeId};

/// Node properties that clips can drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeProperty {
    /// Local translation.
    Translation,
    /// Local rotation.
    Rotation,
    /// Local scale.
    Scale,
    /// Visibility flag.
    Visibility,
}

impl NodeProperty {
    /// Decodes a property discriminant produced by [`Self::as_u8`].
    #[inline]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Translation),
            1 => Some(Self::Rotation),
            2 => Some(Self::Scale),
            3 => Some(Self::Visibility),
            _ => None,
        }
    }
    /// Encodes the property as a stable discriminant.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Bone properties that clips can drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BoneProperty {
    /// Bone local translation.
    Translation,
    /// Bone local rotation.
    Rotation,
    /// Bone local scale.
    Scale,
}

impl BoneProperty {
    /// Decodes a property discriminant.
    #[inline]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Translation),
            1 => Some(Self::Rotation),
            2 => Some(Self::Scale),
            _ => None,
        }
    }
    /// Encodes the property as a stable discriminant.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// PBR material properties that clips can drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MaterialProperty {
    /// Base color (albedo).
    Albedo,
    /// Base color alpha.
    Opacity,
    /// Emissive RGB color.
    Emissive,
    /// Roughness factor.
    Roughness,
    /// Metallic factor.
    Metallic,
}

impl MaterialProperty {
    /// Decodes a property discriminant.
    #[inline]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Albedo),
            1 => Some(Self::Opacity),
            2 => Some(Self::Emissive),
            3 => Some(Self::Roughness),
            4 => Some(Self::Metallic),
            _ => None,
        }
    }
    /// Encodes the property as a stable discriminant.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Camera properties that clips can drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CameraProperty {
    /// Perspective vertical field of view in radians.
    FovY,
    /// Camera position.
    Position,
    /// Camera look target.
    Target,
    /// Camera up vector.
    Up,
    /// Orthographic projection bounds.
    OrthographicBounds,
}

impl CameraProperty {
    /// Decodes a property discriminant.
    #[inline]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::FovY),
            1 => Some(Self::Position),
            2 => Some(Self::Target),
            3 => Some(Self::Up),
            4 => Some(Self::OrthographicBounds),
            _ => None,
        }
    }
    /// Encodes the property as a stable discriminant.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Light properties that clips can drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LightProperty {
    /// Light color.
    Color,
    /// Scalar intensity.
    Intensity,
    /// Maximum range (point/spot only).
    Range,
    /// Spot outer cone half-angle in radians (spot only).
    SpotAngle,
}

impl LightProperty {
    /// Decodes a property discriminant.
    #[inline]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Color),
            1 => Some(Self::Intensity),
            2 => Some(Self::Range),
            3 => Some(Self::SpotAngle),
            _ => None,
        }
    }
    /// Encodes the property as a stable discriminant.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// One animated target, resolved to a concrete scenix resource + field.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PropertyBinding {
    /// A scene node transform or visibility field.
    Node {
        /// Target node.
        node_id: NodeId,
        /// Field to animate.
        property: NodeProperty,
    },
    /// A skeleton bone local transform field.
    Bone {
        /// Target skeleton index in the mixer's skeleton slice.
        skeleton_index: usize,
        /// Target bone index inside that skeleton.
        bone_index: usize,
        /// Field to animate.
        property: BoneProperty,
    },
    /// A PBR material field.
    Material {
        /// Target material.
        material_id: MaterialId,
        /// Field to animate.
        property: MaterialProperty,
    },
    /// A camera field.
    Camera {
        /// Target camera.
        camera_id: CameraId,
        /// Field to animate.
        property: CameraProperty,
    },
    /// A light field.
    Light {
        /// Target light.
        light_id: LightId,
        /// Field to animate.
        property: LightProperty,
    },
    /// One morph target weight on a mesh.
    MorphWeight {
        /// Target mesh.
        mesh_id: MeshId,
        /// Target morph index inside the mesh's weight stack.
        target_index: usize,
    },
}

/// Stable, hashable key derived from a binding for accumulator lookups in the
/// mixer. Two channels that write the same key are blended together.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BindingKey {
    /// Node binding key.
    Node { id: u64, property: u8 },
    /// Bone binding key.
    Bone {
        skeleton: usize,
        bone: usize,
        property: u8,
    },
    /// Material binding key.
    Material { id: u64, property: u8 },
    /// Camera binding key.
    Camera { id: u64, property: u8 },
    /// Light binding key.
    Light { id: u64, property: u8 },
    /// Morph-weight binding key.
    Morph { id: u64, target: usize },
}

impl PropertyBinding {
    /// Returns the stable accumulator key for this binding.
    #[inline]
    pub fn key(&self) -> BindingKey {
        match *self {
            PropertyBinding::Node { node_id, property } => BindingKey::Node {
                id: node_id.get(),
                property: property.as_u8(),
            },
            PropertyBinding::Bone {
                skeleton_index,
                bone_index,
                property,
            } => BindingKey::Bone {
                skeleton: skeleton_index,
                bone: bone_index,
                property: property.as_u8(),
            },
            PropertyBinding::Material {
                material_id,
                property,
            } => BindingKey::Material {
                id: material_id.get(),
                property: property.as_u8(),
            },
            PropertyBinding::Camera {
                camera_id,
                property,
            } => BindingKey::Camera {
                id: camera_id.get(),
                property: property.as_u8(),
            },
            PropertyBinding::Light { light_id, property } => BindingKey::Light {
                id: light_id.get(),
                property: property.as_u8(),
            },
            PropertyBinding::MorphWeight {
                mesh_id,
                target_index,
            } => BindingKey::Morph {
                id: mesh_id.get(),
                target: target_index,
            },
        }
    }
}
