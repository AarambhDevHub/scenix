//! Distance-based LOD band selection for chunked terrain.

use alloc::vec;
use alloc::vec::Vec;

/// One distance band of a [`LodPolicy`]: meshes closer than `max_distance`
/// render at `level` (0 = highest detail).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LodBand {
    /// Chunks farther than this distance (meters) use the next band.
    pub max_distance: f32,
    /// LOD level applied inside this band; `0` keeps full resolution.
    pub level: u32,
}

impl LodBand {
    /// Creates a band; the distance is sanitized by `LodPolicy::new`.
    #[inline]
    pub const fn new(max_distance: f32, level: u32) -> Self {
        Self {
            max_distance,
            level,
        }
    }
}

/// Ordered distance→level table for streaming chunk LOD decisions.
///
/// Bands are validated at construction: strictly increasing distances,
/// clamped distances, and non-negative levels. `select(distance)` picks the
/// first band whose `max_distance` covers the chunk distance; a band list
/// that ends before infinity extends the last level forever.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LodPolicy {
    bands: Vec<LodBand>,
}

impl LodPolicy {
    /// Builds a validated policy; `None` when `bands` is empty or not sorted
    /// by strictly increasing `max_distance`.
    #[must_use]
    pub fn new(bands: &[LodBand]) -> Option<Self> {
        if bands.is_empty() {
            return None;
        }
        let mut sanitized = Vec::with_capacity(bands.len());
        for band in bands {
            let distance = if band.max_distance.is_finite() && band.max_distance > 0.0 {
                band.max_distance
            } else {
                return None;
            };
            let previous_ok = sanitized
                .last()
                .is_none_or(|last: &LodBand| last.max_distance < distance);
            if !previous_ok {
                return None;
            }
            sanitized.push(LodBand {
                max_distance: distance,
                level: band.level,
            });
        }
        Some(Self { bands: sanitized })
    }

    /// A common 3-band default: full detail within 40 m, one step out to
    /// 120 m, coarse beyond.
    #[must_use]
    pub fn default_three_band() -> Self {
        Self {
            bands: vec![
                LodBand::new(40.0, 0),
                LodBand::new(120.0, 1),
                LodBand::new(320.0, 2),
            ],
        }
    }

    /// Returns the immutable band list.
    #[inline]
    pub fn bands(&self) -> &[LodBand] {
        &self.bands
    }

    /// Returns the LOD level for a chunk-distance in meters.
    #[must_use]
    pub fn select(&self, distance: f32) -> u32 {
        let distance = distance.max(0.0);
        for band in &self.bands {
            if distance <= band.max_distance {
                return band.level;
            }
        }
        self.bands.last().map_or(0, |band| band.level.max(1))
    }

    /// Mesh resolution produced by a `base`-vertex chunk at `level`.
    #[must_use]
    #[inline]
    pub fn level_resolution(base: u32, level: u32) -> u32 {
        if base < 2 {
            return 2;
        }
        let halvings = level.min(30);
        if base < 1_u32 << halvings {
            2
        } else {
            base >> halvings
        }
        .max(2)
    }

    /// Number of vertices per axis a `base`-vertex chunk places at `level`.
    #[must_use]
    pub fn level_vertex_count(base: u32, level: u32) -> u32 {
        Self::level_resolution(base, level) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::{LodBand, LodPolicy};

    #[test]
    fn bands_select_in_distance_order() {
        let policy = LodPolicy::new(&[
            LodBand::new(40.0, 0),
            LodBand::new(120.0, 1),
            LodBand::new(320.0, 2),
        ])
        .expect("valid bands");
        assert_eq!(policy.select(0.0), 0);
        assert_eq!(policy.select(40.0), 0);
        assert_eq!(policy.select(40.5), 1);
        assert_eq!(policy.select(250.0), 2);
        assert_eq!(
            policy.select(4_000.0),
            2,
            "beyond ranges keep the last level"
        );
        assert!(policy.bands().len() == 3);
    }

    #[test]
    fn unsorted_or_empty_bands_are_rejected() {
        assert!(LodPolicy::new(&[]).is_none());
        assert!(LodPolicy::new(&[LodBand::new(50.0, 0), LodBand::new(50.0, 0)]).is_none());
        assert!(LodPolicy::new(&[LodBand::new(60.0, 0), LodBand::new(50.0, 0)]).is_none());
        assert!(LodPolicy::new(&[LodBand::new(0.0, 0)]).is_none());
    }

    #[test]
    fn level_resolution_halves_and_clamps() {
        assert_eq!(LodPolicy::level_resolution(32, 0), 32);
        assert_eq!(LodPolicy::level_resolution(32, 1), 16);
        assert_eq!(LodPolicy::level_resolution(32, 5), 2);
        assert_eq!(LodPolicy::level_resolution(32, 6), 2);
        assert_eq!(LodPolicy::level_vertex_count(32, 0), 33);
        assert_eq!(LodPolicy::level_vertex_count(32, 2), 9);
    }
}
