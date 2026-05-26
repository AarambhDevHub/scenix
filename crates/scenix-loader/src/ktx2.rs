use std::fs;
use std::path::Path;

use scenix_core::{LoadError, ScenixError};
use scenix_texture::{Texture2D, TextureFormat};

const IDENTIFIER: [u8; 12] = [
    0xAB, b'K', b'T', b'X', b' ', b'2', b'0', 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];

const VK_FORMAT_R8G8B8A8_UNORM: u32 = 37;
const VK_FORMAT_R8G8B8A8_SRGB: u32 = 43;
const VK_FORMAT_BC7_UNORM_BLOCK: u32 = 145;
const VK_FORMAT_ETC2_R8G8B8A8_UNORM_BLOCK: u32 = 151;
const VK_FORMAT_ASTC_4X4_UNORM_BLOCK: u32 = 157;

/// Loads a supported KTX2 2D texture container into raw texture bytes.
pub fn load(path: impl AsRef<Path>) -> Result<Texture2D, ScenixError> {
    let bytes = fs::read(path.as_ref()).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            LoadError::NotFound
        } else {
            LoadError::Io
        }
    })?;
    load_bytes(&bytes)
}

/// Parses KTX2 bytes for supported uncompressed and block-compressed 2D formats.
pub fn load_bytes(bytes: &[u8]) -> Result<Texture2D, ScenixError> {
    let header = Header::parse(bytes)?;
    if header.pixel_width == 0
        || header.pixel_height == 0
        || header.pixel_depth > 0
        || header.layer_count > 1
        || header.face_count > 1
    {
        return Err(ScenixError::Load(LoadError::UnsupportedFeature));
    }

    let format = map_vk_format(header.vk_format)?;
    let level_count = header.level_count.max(1);
    let mut data = Vec::new();
    for level in 0..level_count {
        let record = LevelIndex::parse(bytes, level as usize)?;
        let start = record.byte_offset as usize;
        let end = start
            .checked_add(record.byte_length as usize)
            .ok_or(ScenixError::Load(LoadError::Parse))?;
        let level_bytes = bytes
            .get(start..end)
            .ok_or(ScenixError::Load(LoadError::Parse))?;
        data.extend_from_slice(level_bytes);
    }

    Texture2D::with_mip_levels(
        header.pixel_width,
        header.pixel_height,
        format,
        data,
        level_count,
    )
    .map_err(ScenixError::from)
}

fn map_vk_format(vk_format: u32) -> Result<TextureFormat, ScenixError> {
    match vk_format {
        VK_FORMAT_R8G8B8A8_UNORM => Ok(TextureFormat::Rgba8Unorm),
        VK_FORMAT_R8G8B8A8_SRGB => Ok(TextureFormat::Rgba8UnormSrgb),
        VK_FORMAT_BC7_UNORM_BLOCK => Ok(TextureFormat::Bc7RgbaUnorm),
        VK_FORMAT_ETC2_R8G8B8A8_UNORM_BLOCK => Ok(TextureFormat::Etc2Rgba8Unorm),
        VK_FORMAT_ASTC_4X4_UNORM_BLOCK => Ok(TextureFormat::Astc4x4RgbaUnorm),
        _ => Err(ScenixError::Load(LoadError::UnsupportedFormat)),
    }
}

#[derive(Clone, Copy, Debug)]
struct Header {
    vk_format: u32,
    pixel_width: u32,
    pixel_height: u32,
    pixel_depth: u32,
    layer_count: u32,
    face_count: u32,
    level_count: u32,
}

impl Header {
    fn parse(bytes: &[u8]) -> Result<Self, ScenixError> {
        if bytes.len() < 104 || bytes.get(..12) != Some(&IDENTIFIER) {
            return Err(ScenixError::Load(LoadError::UnsupportedFormat));
        }
        Ok(Self {
            vk_format: read_u32(bytes, 12)?,
            pixel_width: read_u32(bytes, 20)?,
            pixel_height: read_u32(bytes, 24)?,
            pixel_depth: read_u32(bytes, 28)?,
            layer_count: read_u32(bytes, 32)?,
            face_count: read_u32(bytes, 36)?,
            level_count: read_u32(bytes, 40)?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct LevelIndex {
    byte_offset: u64,
    byte_length: u64,
}

impl LevelIndex {
    fn parse(bytes: &[u8], level: usize) -> Result<Self, ScenixError> {
        let offset = 80 + level * 24;
        Ok(Self {
            byte_offset: read_u64(bytes, offset)?,
            byte_length: read_u64(bytes, offset + 8)?,
        })
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, ScenixError> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or(ScenixError::Load(LoadError::Parse))?;
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, ScenixError> {
    let slice = bytes
        .get(offset..offset + 8)
        .ok_or(ScenixError::Load(LoadError::Parse))?;
    Ok(u64::from_le_bytes(slice.try_into().unwrap()))
}
