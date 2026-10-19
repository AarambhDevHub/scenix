//! Ring-buffer chunk streaming around a moving camera position.
//!
//! Loading and unloading terrain chunks without pops matters most for
//! vehicles and editors; the streamed window is a deterministic function of
//! the current position, so repeated ticks with no movement produce empty
//! diffs and any DIRECTION change produces the same load order.

use alloc::collections::BTreeSet;

/// Chunk index pair used by streaming calculations.
pub type ChunkCoord = (u32, u32);

/// Which chunks should exist around a world position, expressed as loads and
/// unloads for one tick.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamDiff {
    /// Chunk coordinates that entered the radius this tick, sorted.
    pub load: Vec<ChunkCoord>,
    /// Chunk coordinates that left the radius this tick, sorted.
    pub unload: Vec<ChunkCoord>,
}

impl StreamDiff {
    /// Whether this tick needs any streaming work.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.load.is_empty() && self.unload.is_empty()
    }
}

use alloc::vec::Vec;

/// A deterministic ring-of-chunks streaming window.
///
/// The window holds every chunk whose AABB intersects the radius around the
/// tracked world position. Calls to [`ChunkStreamer::poll`] return the exact
/// difference between the requested and loaded sets, so implementations
/// multiplex real geometry generation behind the request.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChunkStreamer {
    /// Count of loaded chunks along each axis (`2 * half + 1^2` at most).
    pub half_extent: u32,
    loaded: BTreeSet<ChunkCoord>,
}

impl ChunkStreamer {
    /// Opens a streaming window of `half_extent` chunks on both sides of
    /// the tracked position (`half_extent == 4` loads a 9x9 window).
    #[inline]
    pub const fn new(half_extent: u32) -> Self {
        Self {
            half_extent,
            loaded: BTreeSet::new(),
        }
    }

    /// Coordinates currently held.
    #[inline]
    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }

    /// Whether one chunk is inside the loaded set.
    #[inline]
    pub fn contains(&self, coord: ChunkCoord) -> bool {
        self.loaded.contains(&coord)
    }

    /// Drops all loaded chunk records (for scene unloads and tests).
    pub fn clear(&mut self) {
        self.loaded.clear();
    }

    /// Recomputes the window around a chunk coordinate and reports loads and
    /// unloads.
    ///
    /// Chunk-coordinates outside the lattice wrap through modular
    /// arithmetic, so streams are cyclical worlds without a seam.
    #[must_use]
    pub fn poll(&mut self, chunk_x: u32, chunk_z: u32, lattice: (u32, u32)) -> StreamDiff {
        let half = self.half_extent;
        let want = Self::window(chunk_x, chunk_z, half, lattice);
        let mut diff = StreamDiff::default();
        for coord in want.iter().copied() {
            if self.loaded.insert(coord) {
                diff.load.push(coord);
            }
        }
        diff.unload.extend(self.loaded.difference(&want).copied());
        if !diff.unload.is_empty() {
            self.loaded.retain(|coord| want.contains(coord));
        }
        diff
    }

    /// The flat chunk window, sorted (deterministic iteration order).
    fn window(chunk_x: u32, chunk_z: u32, half: u32, lattice: (u32, u32)) -> BTreeSet<ChunkCoord> {
        let mut window = BTreeSet::new();
        let radius = (chunk_x as i64, chunk_z as i64);
        let lattice_x = lattice.0.max(1);
        let lattice_z = lattice.1.max(1);
        for dz in -(half as i64)..=(half as i64) {
            for dx in -(half as i64)..=(half as i64) {
                let wrapped_x = Self::wrap(radius.0 + dx, lattice_x);
                let wrapped_z = Self::wrap(radius.1 + dz, lattice_z);
                window.insert((wrapped_x, wrapped_z));
            }
        }
        window
    }

    /// Modular wrap into `[0, lattice)` for positive and negative inputs.
    #[inline]
    const fn wrap(value: i64, lattice: u32) -> u32 {
        let span = lattice as i64;
        let rem = value % span;
        if rem < 0 {
            (rem + span) as u32
        } else {
            rem as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::{ChunkCoord, ChunkStreamer, StreamDiff};

    #[test]
    fn first_poll_loads_and_second_is_a_no_op() {
        let mut streamer = ChunkStreamer::new(1);
        let diff: StreamDiff = streamer.poll(2, 2, (8, 8));
        assert!(diff.unload.is_empty());
        assert_eq!(diff.load.len(), 9, "3x3 window");
        let held: Vec<ChunkCoord> = diff.load.clone();
        assert!(held.contains(&(2, 2)));

        let repoll = streamer.poll(2, 2, (8, 8));
        assert!(repoll.is_empty());
        assert_eq!(streamer.loaded_count(), 9);
    }

    #[test]
    fn moving_center_unloads_behind_coords() {
        let mut streamer = ChunkStreamer::new(1);
        let _ = streamer.poll(2, 2, (8, 8));
        let diff = streamer.poll(3, 2, (8, 8));
        assert_eq!(diff.load.len(), 3, "3 new chunks on the leading edge");
        assert_eq!(diff.unload.len(), 3, "3 chunks fall out behind");
        assert!(!streamer.contains((1, 2)));
        assert!(streamer.contains((4, 2)));
        assert_eq!(streamer.loaded_count(), 9);
    }

    #[test]
    fn lattices_wrap_modularly() {
        let mut streamer = ChunkStreamer::new(1);
        let diff = streamer.poll(0, 0, (4, 4));
        // Half the window wraps to lattice - 1; spans both edges.
        assert!(streamer.contains((3, 0)));
        assert!(streamer.contains((0, 3)));
        assert_eq!(streamer.loaded_count(), 9);
        let load_count = diff.load.len();
        assert_eq!(load_count, 9);

        // Both wraps target the same clamped coordinate.
        let trailer = streamer.poll(3, 3, (4, 4));
        assert!(trailer.load.iter().any(|(x, z)| *x == 2 && *z == 2));
    }
}
