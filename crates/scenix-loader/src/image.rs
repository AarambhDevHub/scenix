use std::io::Cursor;
use std::path::Path;

use scenix_core::{LoadError, ScenixError};
use scenix_texture::{Texture2D, TextureFormat};

/// Decodes a PNG, JPEG, or WebP image into an sRGB RGBA8 texture.
pub fn load(path: impl AsRef<Path>) -> Result<Texture2D, ScenixError> {
    let reader = ::image::ImageReader::open(path.as_ref()).map_err(io_to_load_error)?;
    let image = reader.decode().map_err(|_| LoadError::Decode)?;
    rgba8_texture(image, Some(path.as_ref().to_string_lossy().into_owned()))
}

/// Decodes image bytes into an sRGB RGBA8 texture.
pub fn load_bytes(bytes: &[u8]) -> Result<Texture2D, ScenixError> {
    let image = ::image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| LoadError::UnsupportedFormat)?
        .decode()
        .map_err(|_| LoadError::Decode)?;
    rgba8_texture(image, None)
}

fn rgba8_texture(
    image: ::image::DynamicImage,
    label: Option<String>,
) -> Result<Texture2D, ScenixError> {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut texture = Texture2D::new(
        width,
        height,
        TextureFormat::Rgba8UnormSrgb,
        rgba.into_raw(),
    )?;
    texture.label = label;
    Ok(texture)
}

fn io_to_load_error(err: std::io::Error) -> LoadError {
    if err.kind() == std::io::ErrorKind::NotFound {
        LoadError::NotFound
    } else {
        LoadError::Io
    }
}
