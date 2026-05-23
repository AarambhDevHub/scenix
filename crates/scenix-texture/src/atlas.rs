use alloc::string::String;
use alloc::vec::Vec;

use scenix_core::ValidationError;

/// Pixel-space rectangle inside a texture atlas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AtlasRect {
    /// Left pixel.
    pub x: u32,
    /// Top pixel.
    pub y: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Normalized UV rectangle inside a texture atlas.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UvRect {
    /// Left U coordinate.
    pub u0: f32,
    /// Top V coordinate.
    pub v0: f32,
    /// Right U coordinate.
    pub u1: f32,
    /// Bottom V coordinate.
    pub v1: f32,
}

/// Named atlas entry.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AtlasEntry {
    /// Entry name.
    pub name: String,
    /// Pixel-space rectangle.
    pub rect: AtlasRect,
    /// Normalized UV rectangle.
    pub uv: UvRect,
}

/// Deterministic shelf-packed texture atlas.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextureAtlas {
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Padding inserted between shelves and entries.
    pub padding: u32,
    cursor_x: u32,
    cursor_y: u32,
    shelf_height: u32,
    entries: Vec<AtlasEntry>,
}

impl AtlasRect {
    /// Creates a pixel-space rectangle.
    #[inline]
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl TextureAtlas {
    /// Creates an empty atlas.
    #[inline]
    pub const fn new(width: u32, height: u32) -> Self {
        Self::with_padding(width, height, 0)
    }

    /// Creates an empty atlas with padding.
    #[inline]
    pub const fn with_padding(width: u32, height: u32, padding: u32) -> Self {
        Self {
            width,
            height,
            padding,
            cursor_x: 0,
            cursor_y: 0,
            shelf_height: 0,
            entries: Vec::new(),
        }
    }

    /// Inserts a named rectangle and returns its pixel-space placement.
    pub fn insert(
        &mut self,
        name: impl Into<String>,
        width: u32,
        height: u32,
    ) -> Result<AtlasRect, ValidationError> {
        if self.width == 0 || self.height == 0 || width == 0 || height == 0 {
            return Err(ValidationError::OutOfRange);
        }
        if width > self.width || height > self.height {
            return Err(ValidationError::OutOfRange);
        }

        let name = name.into();
        if self.entries.iter().any(|entry| entry.name == name) {
            return Err(ValidationError::InvalidState);
        }

        if self.cursor_x > 0 && self.cursor_x + width > self.width {
            self.cursor_x = 0;
            self.cursor_y = self
                .cursor_y
                .checked_add(self.shelf_height)
                .and_then(|value| value.checked_add(self.padding))
                .ok_or(ValidationError::OutOfRange)?;
            self.shelf_height = 0;
        }

        if self.cursor_y + height > self.height {
            return Err(ValidationError::OutOfRange);
        }

        let rect = AtlasRect::new(self.cursor_x, self.cursor_y, width, height);
        let uv = self.make_uv(rect);
        self.entries.push(AtlasEntry { name, rect, uv });

        self.cursor_x = self
            .cursor_x
            .checked_add(width)
            .and_then(|value| value.checked_add(self.padding))
            .ok_or(ValidationError::OutOfRange)?;
        self.shelf_height = self.shelf_height.max(height);

        Ok(rect)
    }

    /// Returns a named pixel-space rectangle.
    #[inline]
    pub fn rect(&self, name: &str) -> Option<AtlasRect> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.rect)
    }

    /// Returns a named normalized UV rectangle.
    #[inline]
    pub fn uv(&self, name: &str) -> Option<UvRect> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.uv)
    }

    /// Returns all packed entries in insertion order.
    #[inline]
    pub fn entries(&self) -> &[AtlasEntry] {
        &self.entries
    }

    #[inline]
    fn make_uv(&self, rect: AtlasRect) -> UvRect {
        UvRect {
            u0: rect.x as f32 / self.width as f32,
            v0: rect.y as f32 / self.height as f32,
            u1: (rect.x + rect.width) as f32 / self.width as f32,
            v1: (rect.y + rect.height) as f32 / self.height as f32,
        }
    }
}
