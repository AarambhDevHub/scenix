use alloc::string::String;

#[cfg(feature = "std")]
use scenix_core::Named;
use scenix_core::{CameraId, LightId, MaterialId, MeshId};
use scenix_math::Transform;

use crate::Sprite;

/// Render or logical payload attached to a scene node.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeKind {
    /// No payload.
    #[default]
    Empty,
    /// Logical grouping node.
    Group,
    /// Mesh renderable.
    Mesh {
        /// Mesh resource identifier.
        mesh_id: MeshId,
        /// Material resource identifier.
        material_id: MaterialId,
    },
    /// Light attachment.
    Light {
        /// Light resource identifier.
        light_id: LightId,
    },
    /// Camera attachment.
    Camera {
        /// Camera resource identifier.
        camera_id: CameraId,
    },
    /// Sprite attachment.
    Sprite(Sprite),
}

/// A node's public, user-editable scene data.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SceneNode {
    /// Human-readable node name.
    pub name: String,
    /// Local transform relative to the parent node.
    pub transform: Transform,
    /// Whether this node and its render payload should be considered visible.
    pub visible: bool,
    /// Camera culling layer bitmask.
    pub layer: u32,
    /// Node payload.
    pub kind: NodeKind,
}

impl SceneNode {
    /// Creates an empty node.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transform: Transform::IDENTITY,
            visible: true,
            layer: u32::MAX,
            kind: NodeKind::Empty,
        }
    }

    /// Creates an empty node.
    #[inline]
    pub fn empty(name: impl Into<String>) -> Self {
        Self::new(name)
    }

    /// Creates a logical group node.
    #[inline]
    pub fn group(name: impl Into<String>) -> Self {
        Self::new(name).kind(NodeKind::Group)
    }

    /// Creates a mesh node.
    #[inline]
    pub fn mesh(name: impl Into<String>, mesh_id: MeshId, material_id: MaterialId) -> Self {
        Self::new(name).kind(NodeKind::Mesh {
            mesh_id,
            material_id,
        })
    }

    /// Creates a light node.
    #[inline]
    pub fn light(name: impl Into<String>, light_id: LightId) -> Self {
        Self::new(name).kind(NodeKind::Light { light_id })
    }

    /// Creates a camera node.
    #[inline]
    pub fn camera(name: impl Into<String>, camera_id: CameraId) -> Self {
        Self::new(name).kind(NodeKind::Camera { camera_id })
    }

    /// Creates a sprite node.
    #[inline]
    pub fn sprite(name: impl Into<String>, sprite: Sprite) -> Self {
        Self::new(name).kind(NodeKind::Sprite(sprite))
    }

    /// Returns this node with a local transform.
    #[inline]
    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Returns this node with visibility set.
    #[inline]
    pub const fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Returns this node with a layer bitmask.
    #[inline]
    pub const fn layer(mut self, layer: u32) -> Self {
        self.layer = layer;
        self
    }

    /// Returns this node with a payload kind.
    #[inline]
    pub fn kind(mut self, kind: NodeKind) -> Self {
        self.kind = kind;
        self
    }
}

impl Default for SceneNode {
    #[inline]
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(feature = "std")]
impl Named for SceneNode {
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
}
