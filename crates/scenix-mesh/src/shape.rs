use alloc::vec::Vec;

use scenix_math::Vec2;

/// A 2D polygon shape with an exterior contour and optional holes.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Shape {
    contours: Vec<Vec<Vec2>>,
}

impl Shape {
    /// Creates a shape from an exterior contour.
    #[inline]
    pub fn new(exterior: Vec<Vec2>) -> Self {
        Self {
            contours: alloc::vec![exterior],
        }
    }

    /// Creates a shape from an exterior contour and holes.
    #[inline]
    pub fn with_holes(exterior: Vec<Vec2>, holes: Vec<Vec<Vec2>>) -> Self {
        let mut contours = Vec::with_capacity(holes.len() + 1);
        contours.push(exterior);
        contours.extend(holes);
        Self { contours }
    }

    /// Adds a hole contour.
    #[inline]
    pub fn add_hole(&mut self, hole: Vec<Vec2>) {
        self.contours.push(hole);
    }

    /// Returns the exterior contour, if present.
    #[inline]
    pub fn exterior(&self) -> Option<&[Vec2]> {
        self.contours.first().map(Vec::as_slice)
    }

    /// Returns the hole contours.
    #[inline]
    pub fn holes(&self) -> &[Vec<Vec2>] {
        if self.contours.len() <= 1 {
            &[]
        } else {
            &self.contours[1..]
        }
    }

    /// Returns all contours, with the exterior first.
    #[inline]
    pub fn contours(&self) -> &[Vec<Vec2>] {
        &self.contours
    }

    /// Returns whether the shape has fewer than three exterior points.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.exterior().is_none_or(|points| points.len() < 3)
    }
}
