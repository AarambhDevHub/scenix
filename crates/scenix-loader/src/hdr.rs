use std::path::Path;

use scenix_core::{LoadError, ScenixError};
use scenix_texture::{TextureCube, TextureFormat};

/// Default cube-face size used by [`load`].
pub const DEFAULT_CUBE_SIZE: u32 = 256;

/// Loads an HDR or EXR image and converts it to a simple RGBA8 cube texture.
pub fn load(path: impl AsRef<Path>) -> Result<TextureCube, ScenixError> {
    load_with_size(path, DEFAULT_CUBE_SIZE)
}

/// Loads an HDR or EXR image and converts it to a cube texture with `size` texels per face.
pub fn load_with_size(path: impl AsRef<Path>, size: u32) -> Result<TextureCube, ScenixError> {
    if size == 0 {
        return Err(ScenixError::Load(LoadError::UnsupportedFeature));
    }

    let image = ::image::open(path.as_ref()).map_err(|_| LoadError::Decode)?;
    let rgba = image.to_rgba8();
    let average = average_rgba(&rgba);
    let face_len = TextureFormat::Rgba8Unorm
        .expected_2d_len(size, size)
        .map_err(ScenixError::from)?;
    let face = repeat_rgba(average, face_len);
    TextureCube::new(
        size,
        TextureFormat::Rgba8Unorm,
        [
            face.clone(),
            face.clone(),
            face.clone(),
            face.clone(),
            face.clone(),
            face,
        ],
    )
    .map_err(ScenixError::from)
}

fn average_rgba(image: &::image::RgbaImage) -> [u8; 4] {
    let mut sum = [0_u64; 4];
    let pixels = image.as_raw();
    if pixels.is_empty() {
        return [0, 0, 0, 255];
    }
    for rgba in pixels.chunks_exact(4) {
        sum[0] += rgba[0] as u64;
        sum[1] += rgba[1] as u64;
        sum[2] += rgba[2] as u64;
        sum[3] += rgba[3] as u64;
    }
    let count = (pixels.len() / 4) as u64;
    [
        (sum[0] / count) as u8,
        (sum[1] / count) as u8,
        (sum[2] / count) as u8,
        (sum[3] / count) as u8,
    ]
}

fn repeat_rgba(rgba: [u8; 4], len: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(len);
    for _ in 0..(len / 4) {
        data.extend_from_slice(&rgba);
    }
    data
}
