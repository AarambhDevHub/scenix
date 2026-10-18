//! Flipbook grid animation for particle atlases.
//!
//! A flipbook treats a texture as a uniform grid of animation frames. The
//! CPU simulator maps every particle's age to one grid cell and emits the
//! texture-space uv rectangle with the render batch, so the GPU draw path
//! never performs frame math.

/// Grid animation parameters for one particle texture.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlipbookConfig {
    /// Meta grid columns.
    pub columns: u32,
    /// Meta grid rows.
    pub rows: u32,
    /// Frames rendered before the animation stops; `0` selects the full grid.
    pub frame_count: u32,
    /// Frames per second.
    pub fps: f32,
    /// Restart the animation after the last frame.
    pub looping: bool,
}

impl FlipbookConfig {
    /// Creates a looping full-grid flipbook.
    #[inline]
    pub const fn full(columns: u32, rows: u32, fps: f32) -> Self {
        Self {
            columns,
            rows,
            frame_count: 0,
            fps,
            looping: true,
        }
    }

    /// Returns the effective (clamped) frame count.
    #[inline]
    pub const fn effective_frame_count(&self) -> u32 {
        let grid = self.columns * self.rows;
        if grid == 0 {
            0
        } else if self.frame_count == 0 || self.frame_count > grid {
            grid
        } else {
            self.frame_count
        }
    }

    /// Returns the frame index for a particle age in seconds.
    #[inline]
    pub fn frame_at(&self, age: f32) -> u32 {
        let frames = self.effective_frame_count();
        if frames == 0 || self.fps <= 0.0 {
            return 0;
        }
        let frame = (age * self.fps.max(0.0)) as u32;
        if self.looping {
            frame % frames
        } else {
            frame.min(frames - 1)
        }
    }

    /// Returns the texture-space uv rect for a frame index as
    /// `[u0, v0, u1, v1]` with a top-left uv origin.
    ///
    /// Out-of-range indices wrap modulo the effective frame count.
    #[inline]
    pub const fn uv_rect(&self, frame: u32) -> [f32; 4] {
        let frames = self.effective_frame_count();
        if frames == 0 || self.columns == 0 || self.rows == 0 {
            return [0.0, 0.0, 1.0, 1.0];
        }
        let frame = frame % frames;
        let column = frame % self.columns;
        let row = frame / self.columns;
        let column_width = 1.0 / self.columns as f32;
        let row_height = 1.0 / self.rows as f32;
        [
            column as f32 * column_width,
            row as f32 * row_height,
            (column + 1) as f32 * column_width,
            (row + 1) as f32 * row_height,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_grid_uses_every_cell_in_order() {
        let flip = FlipbookConfig::full(2, 2, 4.0);
        assert_eq!(flip.effective_frame_count(), 4);
        assert_eq!(flip.frame_at(0.0), 0);
        assert_eq!(flip.frame_at(0.3), 1);
        assert_eq!(flip.frame_at(0.6), 2);
        assert_eq!(flip.frame_at(0.95), 3);
        assert_eq!(flip.frame_at(1.05), 0, "looping restarts");
    }

    #[test]
    fn non_looping_flipbooks_stop_on_the_last_frame() {
        let flip = FlipbookConfig {
            columns: 2,
            rows: 2,
            frame_count: 3,
            fps: 10.0,
            looping: false,
        };
        assert_eq!(flip.effective_frame_count(), 3);
        assert_eq!(flip.frame_at(0.0), 0);
        assert_eq!(flip.frame_at(0.5), 2);
        assert_eq!(flip.frame_at(9.0), 2, "clamped at last frame");
    }

    #[test]
    fn uv_rects_tile_the_grid() {
        let flip = FlipbookConfig::full(4, 2, 8.0);
        let frame0 = flip.uv_rect(0);
        assert_eq!(frame0, [0.0, 0.0, 0.25, 0.5]);
        let frame5 = flip.uv_rect(5);
        assert_eq!(frame5, [0.25, 0.5, 0.5, 1.0]);
        assert_eq!(flip.uv_rect(9), flip.uv_rect(1), "wraps modulo");
    }

    #[test]
    fn degenerate_grids_cover_the_full_texture() {
        let flip = FlipbookConfig {
            columns: 0,
            rows: 0,
            frame_count: 4,
            fps: 8.0,
            looping: true,
        };
        assert_eq!(flip.effective_frame_count(), 0);
        assert_eq!(flip.uv_rect(2), [0.0, 0.0, 1.0, 1.0]);
    }
}
