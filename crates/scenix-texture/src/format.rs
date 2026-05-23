use scenix_core::ValidationError;

/// CPU-side texture format metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextureFormat {
    /// Four 8-bit normalized linear RGBA channels.
    Rgba8Unorm,
    /// Four 8-bit normalized sRGB RGBA channels.
    Rgba8UnormSrgb,
    /// Four 16-bit floating-point RGBA channels.
    Rgba16Float,
    /// 32-bit floating-point depth.
    Depth32Float,
    /// BC7 compressed RGBA, 4x4 blocks, 16 bytes per block.
    Bc7RgbaUnorm,
    /// ASTC 4x4 compressed RGBA, 16 bytes per block.
    Astc4x4RgbaUnorm,
    /// ETC2 RGBA8 compressed data, 4x4 blocks, 16 bytes per block.
    Etc2Rgba8Unorm,
}

impl TextureFormat {
    /// Returns whether this is a block-compressed format.
    #[inline]
    pub const fn is_compressed(self) -> bool {
        matches!(
            self,
            Self::Bc7RgbaUnorm | Self::Astc4x4RgbaUnorm | Self::Etc2Rgba8Unorm
        )
    }

    /// Returns bytes per texel for uncompressed formats.
    #[inline]
    pub const fn bytes_per_pixel(self) -> Option<usize> {
        match self {
            Self::Rgba8Unorm | Self::Rgba8UnormSrgb | Self::Depth32Float => Some(4),
            Self::Rgba16Float => Some(8),
            Self::Bc7RgbaUnorm | Self::Astc4x4RgbaUnorm | Self::Etc2Rgba8Unorm => None,
        }
    }

    /// Returns `(width, height)` for compressed blocks.
    #[inline]
    pub const fn block_dimensions(self) -> Option<(u32, u32)> {
        if self.is_compressed() {
            Some((4, 4))
        } else {
            None
        }
    }

    /// Returns bytes per compressed block.
    #[inline]
    pub const fn bytes_per_block(self) -> Option<usize> {
        if self.is_compressed() { Some(16) } else { None }
    }

    /// Returns the dimensions of a mip level.
    #[inline]
    pub const fn mip_dimensions(width: u32, height: u32, level: u32) -> (u32, u32) {
        let w = shr_clamped(width, level);
        let h = shr_clamped(height, level);
        (if w == 0 { 1 } else { w }, if h == 0 { 1 } else { h })
    }

    /// Returns the expected byte length for a 2D texture level.
    #[inline]
    pub fn expected_2d_len(self, width: u32, height: u32) -> Result<usize, ValidationError> {
        self.expected_3d_len(width, height, 1)
    }

    /// Returns the expected byte length for a 3D texture level.
    pub fn expected_3d_len(
        self,
        width: u32,
        height: u32,
        depth: u32,
    ) -> Result<usize, ValidationError> {
        if width == 0 || height == 0 || depth == 0 {
            return Err(ValidationError::OutOfRange);
        }

        if let Some(bytes_per_pixel) = self.bytes_per_pixel() {
            checked_area(width, height, depth)?
                .checked_mul(bytes_per_pixel)
                .ok_or(ValidationError::OutOfRange)
        } else {
            let (block_w, block_h) = self.block_dimensions().unwrap_or((4, 4));
            let blocks_x = width.div_ceil(block_w);
            let blocks_y = height.div_ceil(block_h);
            checked_area(blocks_x, blocks_y, depth)?
                .checked_mul(self.bytes_per_block().unwrap_or(16))
                .ok_or(ValidationError::OutOfRange)
        }
    }
}

#[inline]
const fn shr_clamped(value: u32, shift: u32) -> u32 {
    if shift >= u32::BITS {
        0
    } else {
        value >> shift
    }
}

#[inline]
fn checked_area(width: u32, height: u32, depth: u32) -> Result<usize, ValidationError> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|value| value.checked_mul(depth as usize))
        .ok_or(ValidationError::OutOfRange)
}
