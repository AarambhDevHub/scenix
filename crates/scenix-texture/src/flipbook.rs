//! Grid flipbook sheet layouts shared by particle and sprite systems.
//!
//! A flipbook sheet packs animation frames into a uniform `columns × rows`
//! grid; frame *n* occupies column `n % columns`, row `n / columns`. Frames
//! are indexed left-to-right, top-to-bottom (row 0 is the top of the image),
//! matching how flipbook sheets are usually authored in art tools.

use alloc::vec::Vec;

use crate::atlas::UvRect;

/// Uniform grid flipbook layout over one texture.
///
/// `FlipbookGrid` stores no pixels — it is the pure frame-index → UV math
/// used both for CPU atlas generation and for GPU particle flipbooks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlipbookGrid {
    /// Number of columns across the sheet.
    pub columns: u32,
    /// Number of rows down the sheet.
    pub rows: u32,
}

impl FlipbookGrid {
    /// Creates a grid layout. `None` if either count is zero.
    #[inline]
    pub const fn new(columns: u32, rows: u32) -> Option<Self> {
        if columns == 0 || rows == 0 {
            None
        } else {
            Some(Self { columns, rows })
        }
    }

    /// Total frame count (`columns * rows`).
    #[inline]
    pub const fn frame_count(&self) -> u32 {
        self.columns * self.rows
    }

    /// Column/row of `frame`, clamped into the sheet.
    #[inline]
    pub const fn frame_position(&self, frame: u32) -> (u32, u32) {
        let frame = if frame >= self.frame_count() {
            self.frame_count() - 1
        } else {
            frame
        };
        (frame % self.columns, frame / self.columns)
    }

    /// Normalized UV rectangle of `frame`, clamped into the sheet.
    ///
    /// Row 0 is the *top* of the sheet: `v0 < v1` because UV space inside
    /// scenix has `v0` at the top edge of the image.
    #[must_use]
    #[inline]
    pub fn uv(&self, frame: u32) -> UvRect {
        let (column, row) = self.frame_position(frame);
        let cell_u = 1.0 / self.columns as f32;
        let cell_v = 1.0 / self.rows as f32;
        UvRect {
            u0: column as f32 * cell_u,
            v0: row as f32 * cell_v,
            u1: (column + 1) as f32 * cell_u,
            v1: (row + 1) as f32 * cell_v,
        }
    }

    /// UV rectangles of every frame in playback order.
    #[must_use]
    pub fn uv_frames(&self) -> Vec<UvRect> {
        (0..self.frame_count())
            .map(|frame| self.uv(frame))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::FlipbookGrid;

    #[test]
    fn grid_maps_frames_in_reading_order() {
        let Some(grid) = FlipbookGrid::new(4, 2) else {
            panic!("valid grid");
        };
        assert_eq!(grid.frame_count(), 8);

        let first = grid.uv(0);
        assert!((first.u0 - 0.0).abs() < 1.0e-6);
        assert!((first.v0 - 0.0).abs() < 1.0e-6, "row 0 is top");

        let right_after = grid.uv(3);
        assert!(
            (right_after.v0 - 0.0).abs() < 1.0e-6,
            "frame 3 still top row"
        );

        let second_row = grid.uv(4);
        assert!((second_row.v0 - 0.5).abs() < 1.0e-6, "frame 4 starts row 1");

        let last = grid.uv(7);
        assert!((last.u1 - 1.0).abs() < 1.0e-6);
        assert!((last.v1 - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn out_of_range_frames_clamp_to_the_last_cell() {
        let Some(grid) = FlipbookGrid::new(2, 2) else {
            panic!("valid grid");
        };
        let clamped = grid.uv(99);
        let last = grid.uv(3);
        assert_eq!(clamped, last);
    }

    #[test]
    fn zero_counts_are_rejected() {
        assert!(FlipbookGrid::new(0, 4).is_none());
        assert!(FlipbookGrid::new(4, 0).is_none());
    }
}
