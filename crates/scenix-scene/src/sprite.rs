use scenix_core::TextureId;

/// How a sprite should face the camera.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BillboardMode {
    /// Sprite keeps its node rotation.
    #[default]
    None,
    /// Sprite faces the active camera.
    FaceCamera,
    /// Sprite rotates around the world Y axis to face the active camera.
    AxisAlignedY,
}

/// CPU-side sprite attachment data.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sprite {
    /// Sprite width in local units.
    pub width: f32,
    /// Sprite height in local units.
    pub height: f32,
    /// Texture displayed by the sprite.
    pub texture_id: TextureId,
    /// Billboard facing behavior.
    pub billboard: BillboardMode,
}

impl Sprite {
    /// Creates a sprite with no billboard rotation.
    #[inline]
    pub const fn new(width: f32, height: f32, texture_id: TextureId) -> Self {
        Self {
            width,
            height,
            texture_id,
            billboard: BillboardMode::None,
        }
    }

    /// Returns this sprite with a billboard mode.
    #[inline]
    pub const fn billboard(mut self, billboard: BillboardMode) -> Self {
        self.billboard = billboard;
        self
    }
}
