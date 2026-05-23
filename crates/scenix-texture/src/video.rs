use scenix_core::ValidationError;

use crate::Texture2D;

/// Mutable CPU-side texture updated frame by frame.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VideoTexture {
    /// Current texture frame.
    pub texture: Texture2D,
    /// Number of successfully uploaded frames.
    pub frame_index: u64,
    /// Whether the texture has changed since the last upload.
    pub dirty: bool,
}

impl VideoTexture {
    /// Creates a video texture from an initial frame.
    #[inline]
    pub fn new(texture: Texture2D) -> Self {
        Self {
            texture,
            frame_index: 0,
            dirty: true,
        }
    }

    /// Replaces the current frame and marks the texture dirty.
    pub fn update_frame(&mut self, data: &[u8]) -> Result<(), ValidationError> {
        if data.len() != self.texture.base_level_len()? {
            return Err(ValidationError::OutOfRange);
        }
        self.texture.data.clear();
        self.texture.data.extend_from_slice(data);
        self.frame_index = self.frame_index.saturating_add(1);
        self.dirty = true;
        Ok(())
    }

    /// Clears the dirty flag.
    #[inline]
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}
