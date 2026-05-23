use alloc::string::String;
use alloc::vec::Vec;

use scenix_core::ValidationError;

use crate::TextureFormat;

/// CPU-side 2D texture bytes.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Texture2D {
    /// Width in texels.
    pub width: u32,
    /// Height in texels.
    pub height: u32,
    /// Texture format.
    pub format: TextureFormat,
    /// Contiguous texture bytes.
    pub data: Vec<u8>,
    /// Number of mip levels. `0` means base data with auto generation later.
    pub mip_levels: u32,
    /// Optional debug label.
    pub label: Option<String>,
}

/// CPU-side cube texture bytes.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextureCube {
    /// Face width and height in texels.
    pub size: u32,
    /// Texture format.
    pub format: TextureFormat,
    /// Six faces in positive X, negative X, positive Y, negative Y, positive Z, negative Z order.
    pub faces: [Vec<u8>; 6],
    /// Number of mip levels per face.
    pub mip_levels: u32,
    /// Optional debug label.
    pub label: Option<String>,
}

/// CPU-side 3D texture bytes.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Texture3D {
    /// Width in texels.
    pub width: u32,
    /// Height in texels.
    pub height: u32,
    /// Depth in texels.
    pub depth: u32,
    /// Texture format.
    pub format: TextureFormat,
    /// Contiguous texture bytes.
    pub data: Vec<u8>,
    /// Number of mip levels. `0` means base data with auto generation later.
    pub mip_levels: u32,
    /// Optional debug label.
    pub label: Option<String>,
}

impl Texture2D {
    /// Creates a base-level texture and validates byte size.
    #[inline]
    pub fn new(
        width: u32,
        height: u32,
        format: TextureFormat,
        data: Vec<u8>,
    ) -> Result<Self, ValidationError> {
        Self::with_mip_levels(width, height, format, data, 1)
    }

    /// Creates a texture with explicit mip-level count and validates byte size.
    pub fn with_mip_levels(
        width: u32,
        height: u32,
        format: TextureFormat,
        data: Vec<u8>,
        mip_levels: u32,
    ) -> Result<Self, ValidationError> {
        let texture = Self {
            width,
            height,
            format,
            data,
            mip_levels,
            label: None,
        };
        texture.validate()?;
        Ok(texture)
    }

    /// Creates a texture from explicit mip levels and flattens the data.
    pub fn from_mips(
        width: u32,
        height: u32,
        format: TextureFormat,
        mips: Vec<Vec<u8>>,
    ) -> Result<Self, ValidationError> {
        if mips.is_empty() {
            return Err(ValidationError::OutOfRange);
        }
        validate_2d_mips(format, width, height, &mips)?;

        let mip_levels = mips.len() as u32;
        let total_len = mips.iter().try_fold(0_usize, |total, mip| {
            total
                .checked_add(mip.len())
                .ok_or(ValidationError::OutOfRange)
        })?;
        let mut data = Vec::with_capacity(total_len);
        for mip in mips {
            data.extend_from_slice(&mip);
        }

        Ok(Self {
            width,
            height,
            format,
            data,
            mip_levels,
            label: None,
        })
    }

    /// Returns this texture with a label.
    #[inline]
    pub fn labeled(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Returns the expected byte length of the base mip level.
    #[inline]
    pub fn base_level_len(&self) -> Result<usize, ValidationError> {
        self.format.expected_2d_len(self.width, self.height)
    }

    /// Validates dimensions and byte length.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let expected =
            expected_2d_len_for_mip_count(self.format, self.width, self.height, self.mip_levels)?;
        if self.data.len() == expected {
            Ok(())
        } else {
            Err(ValidationError::OutOfRange)
        }
    }
}

impl TextureCube {
    /// Creates a cube texture with six faces and validates each face.
    pub fn new(
        size: u32,
        format: TextureFormat,
        faces: [Vec<u8>; 6],
    ) -> Result<Self, ValidationError> {
        let texture = Self {
            size,
            format,
            faces,
            mip_levels: 1,
            label: None,
        };
        texture.validate()?;
        Ok(texture)
    }

    /// Returns this cube texture with a label.
    #[inline]
    pub fn labeled(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Validates dimensions and every face byte length.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let expected =
            expected_2d_len_for_mip_count(self.format, self.size, self.size, self.mip_levels)?;
        if self.faces.iter().all(|face| face.len() == expected) {
            Ok(())
        } else {
            Err(ValidationError::OutOfRange)
        }
    }
}

impl Texture3D {
    /// Creates a base-level 3D texture and validates byte size.
    #[inline]
    pub fn new(
        width: u32,
        height: u32,
        depth: u32,
        format: TextureFormat,
        data: Vec<u8>,
    ) -> Result<Self, ValidationError> {
        Self::with_mip_levels(width, height, depth, format, data, 1)
    }

    /// Creates a 3D texture with explicit mip-level count and validates byte size.
    pub fn with_mip_levels(
        width: u32,
        height: u32,
        depth: u32,
        format: TextureFormat,
        data: Vec<u8>,
        mip_levels: u32,
    ) -> Result<Self, ValidationError> {
        let texture = Self {
            width,
            height,
            depth,
            format,
            data,
            mip_levels,
            label: None,
        };
        texture.validate()?;
        Ok(texture)
    }

    /// Returns this texture with a label.
    #[inline]
    pub fn labeled(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Validates dimensions and byte length.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let levels = self.mip_levels.max(1);
        if levels > max_mip_levels_3d(self.width, self.height, self.depth)? {
            return Err(ValidationError::OutOfRange);
        }
        let mut expected = 0_usize;
        for level in 0..levels {
            let (width, height) = TextureFormat::mip_dimensions(self.width, self.height, level);
            let depth = mip_dimension(self.depth, level);
            expected = expected
                .checked_add(self.format.expected_3d_len(width, height, depth)?)
                .ok_or(ValidationError::OutOfRange)?;
        }
        if self.data.len() == expected {
            Ok(())
        } else {
            Err(ValidationError::OutOfRange)
        }
    }
}

fn validate_2d_mips(
    format: TextureFormat,
    width: u32,
    height: u32,
    mips: &[Vec<u8>],
) -> Result<(), ValidationError> {
    if mips.len() > max_mip_levels_2d(width, height)? as usize {
        return Err(ValidationError::OutOfRange);
    }
    for (level, mip) in mips.iter().enumerate() {
        let (w, h) = TextureFormat::mip_dimensions(width, height, level as u32);
        if mip.len() != format.expected_2d_len(w, h)? {
            return Err(ValidationError::OutOfRange);
        }
    }
    Ok(())
}

fn expected_2d_len_for_mip_count(
    format: TextureFormat,
    width: u32,
    height: u32,
    mip_levels: u32,
) -> Result<usize, ValidationError> {
    let levels = mip_levels.max(1);
    if levels > max_mip_levels_2d(width, height)? {
        return Err(ValidationError::OutOfRange);
    }
    let mut expected = 0_usize;
    for level in 0..levels {
        let (w, h) = TextureFormat::mip_dimensions(width, height, level);
        expected = expected
            .checked_add(format.expected_2d_len(w, h)?)
            .ok_or(ValidationError::OutOfRange)?;
    }
    Ok(expected)
}

fn mip_dimension(value: u32, level: u32) -> u32 {
    if level >= u32::BITS {
        1
    } else {
        (value >> level).max(1)
    }
}

fn max_mip_levels_2d(width: u32, height: u32) -> Result<u32, ValidationError> {
    let max_dimension = width.max(height);
    if max_dimension == 0 {
        Err(ValidationError::OutOfRange)
    } else {
        Ok(u32::BITS - max_dimension.leading_zeros())
    }
}

fn max_mip_levels_3d(width: u32, height: u32, depth: u32) -> Result<u32, ValidationError> {
    let max_dimension = width.max(height).max(depth);
    if max_dimension == 0 {
        Err(ValidationError::OutOfRange)
    } else {
        Ok(u32::BITS - max_dimension.leading_zeros())
    }
}
