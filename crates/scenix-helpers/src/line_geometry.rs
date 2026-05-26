use alloc::vec::Vec;

use scenix_core::{Color, ValidationError};
use scenix_math::{Aabb, Vec3};

/// A validated line-list geometry used by debug helpers.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineGeometry {
    /// Vertex positions.
    pub positions: Vec<Vec3>,
    /// Per-vertex colors. Empty means the consumer should use a fallback color.
    pub colors: Vec<Color>,
    /// Optional line-list indices. When empty, positions are consumed as pairs.
    pub indices: Vec<u32>,
}

impl LineGeometry {
    /// Creates empty line geometry.
    #[inline]
    pub const fn new() -> Self {
        Self {
            positions: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Returns the number of vertices.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns the number of line segments.
    #[inline]
    pub fn segment_count(&self) -> usize {
        if self.indices.is_empty() {
            self.positions.len() / 2
        } else {
            self.indices.len() / 2
        }
    }

    /// Returns whether no positions are stored.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Adds a segment with one color on both endpoints.
    #[inline]
    pub fn push_segment(&mut self, start: Vec3, end: Vec3, color: Color) {
        self.push_colored_segment(start, end, color, color);
    }

    /// Adds a segment with independent endpoint colors.
    pub fn push_colored_segment(
        &mut self,
        start: Vec3,
        end: Vec3,
        start_color: Color,
        end_color: Color,
    ) {
        self.positions.push(start);
        self.positions.push(end);
        self.colors.push(start_color);
        self.colors.push(end_color);
    }

    /// Appends another line geometry and offsets indices when present.
    pub fn merge(&mut self, other: &Self) {
        let base = self.positions.len();
        let incoming = other.positions.len();
        let use_indices = !self.indices.is_empty() || !other.indices.is_empty();

        if use_indices && self.indices.is_empty() {
            self.indices.reserve(base);
            for index in 0..base {
                self.indices.push(index as u32);
            }
        }

        if self.colors.is_empty() && !other.colors.is_empty() {
            self.colors.resize(base, Color::WHITE);
        }
        if !self.colors.is_empty() {
            if other.colors.is_empty() {
                self.colors
                    .extend(core::iter::repeat_n(Color::WHITE, incoming));
            } else {
                self.colors.extend_from_slice(&other.colors);
            }
        }

        self.positions.extend_from_slice(&other.positions);

        if use_indices {
            if other.indices.is_empty() {
                for index in 0..incoming {
                    self.indices.push((base + index) as u32);
                }
            } else {
                for index in &other.indices {
                    self.indices.push(index + base as u32);
                }
            }
        }
    }

    /// Validates color lengths, line arity, and index ranges.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let vertices = self.positions.len();
        if !self.colors.is_empty() && self.colors.len() != vertices {
            return Err(ValidationError::InvalidState);
        }
        if self.indices.is_empty() {
            if !vertices.is_multiple_of(2) {
                return Err(ValidationError::InvalidState);
            }
        } else {
            if !self.indices.len().is_multiple_of(2) {
                return Err(ValidationError::InvalidState);
            }
            for index in &self.indices {
                if *index as usize >= vertices {
                    return Err(ValidationError::OutOfRange);
                }
            }
        }
        Ok(())
    }

    /// Returns the bounds of the line positions.
    #[inline]
    pub fn aabb(&self) -> Aabb {
        Aabb::from_points(&self.positions)
    }
}
