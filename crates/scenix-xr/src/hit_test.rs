//! Real-world hit tests: probe requests and deterministic results.

use alloc::vec::Vec;

use scenix_math::{Ray3, Vec3};

/// Maximum hit results retained from one probe frame.
pub const XR_HIT_MAX_RESULTS: usize = 4;

/// What kind of real-world surface was struck by a probe.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum XrHitSurface {
    /// Struck a detected plane (floor, wall, table, ...).
    Plane,
    /// Struck a persistent anchor maintained by the runtime.
    Anchor,
    /// Struck a depth-based point cloud mesh or reconstructed surface.
    #[default]
    Mesh,
    /// Hit nothing before the probe's range ran out.
    Missed,
}

impl XrHitSurface {
    /// Returns whether a locatable surface was actually struck.
    #[inline]
    #[must_use]
    pub const fn is_hit(self) -> bool {
        !matches!(self, Self::Missed)
    }
}

/// A probe request against the real world.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrHitTestProbe {
    /// Origin of the probe ray in tracking space.
    pub origin: Vec3,
    /// Unit-length direction of the probe ray.
    pub direction: Vec3,
    /// Maximum travel distance in meters; `0` uses the runtime default.
    pub max_distance: f32,
    /// Whether detected planes are candidates.
    pub request_planes: bool,
    /// Whether persistent anchors are candidates.
    pub request_anchors: bool,
    /// Whether detected meshes are candidates.
    pub request_meshes: bool,
}

impl XrHitTestProbe {
    /// Creates a probe from an unnormalized direction; the direction is
    /// normalized during construction.
    #[must_use]
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: normalize_direction(direction),
            max_distance: 0.0,
            request_planes: true,
            request_anchors: true,
            request_meshes: true,
        }
    }

    /// Returns the probe with a new range cap in meters.
    #[inline]
    #[must_use]
    pub const fn with_max_distance(mut self, meters: f32) -> Self {
        self.max_distance = meters.max(0.0);
        self
    }

    /// Returns the probe restricted to the enabled surface kinds.
    #[inline]
    #[must_use]
    pub const fn with_surfaces(mut self, planes: bool, anchors: bool, meshes: bool) -> Self {
        self.request_planes = planes;
        self.request_anchors = anchors;
        self.request_meshes = meshes;
        self
    }

    /// Returns the probe's ray.
    #[inline]
    #[must_use]
    pub fn ray(&self) -> Ray3 {
        Ray3::new(self.origin, self.direction)
    }

    /// Resolves candidate hits against these results, producing a
    /// distance-sorted, filtered list.
    ///
    /// Candidate ordering is deterministic: closer hits come first, and
    /// ties break by surface priority (anchor > plane > mesh).
    #[must_use]
    pub fn resolve(&self, candidates: &[XrHit]) -> Vec<XrHit> {
        let mut hits: Vec<XrHit> = candidates
            .iter()
            .copied()
            .filter(|hit| hit.distance >= 0.0 && self.allows(hit.surface))
            .collect();
        hits.sort_unstable_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then(b.rank().total_cmp(&a.rank()))
        });
        hits.truncate(XR_HIT_MAX_RESULTS);
        hits
    }

    #[inline]
    #[must_use]
    const fn allows(&self, surface: XrHitSurface) -> bool {
        matches!(
            (
                surface,
                self.request_planes,
                self.request_anchors,
                self.request_meshes
            ),
            (XrHitSurface::Plane, true, _, _)
                | (XrHitSurface::Anchor, _, true, _)
                | (XrHitSurface::Mesh, _, _, true)
        )
    }
}

/// One resolved hit against the real world.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrHit {
    /// Distance along the probe ray in meters.
    pub distance: f32,
    /// Surface normal at the hit, tracking space.
    pub normal: Vec3,
    /// Kind of surface struck.
    pub surface: XrHitSurface,
}

impl XrHit {
    /// Creates a hit from distance and normal.
    #[inline]
    #[must_use]
    pub const fn new(distance: f32, normal: Vec3, surface: XrHitSurface) -> Self {
        Self {
            distance,
            normal,
            surface,
        }
    }

    /// World-space hit position given the probe.
    #[inline]
    #[must_use]
    pub fn position(&self, probe: &XrHitTestProbe) -> Vec3 {
        probe.origin + probe.direction * self.distance
    }

    /// Surface priority for deterministic tie-breaks: anchor > plane > mesh.
    #[inline]
    #[must_use]
    pub const fn rank(&self) -> f32 {
        match self.surface {
            XrHitSurface::Anchor => 3.0,
            XrHitSurface::Plane => 2.0,
            XrHitSurface::Mesh => 1.0,
            XrHitSurface::Missed => 0.0,
        }
    }
}

/// `Vec3::normalize` is `core`-stable, but zero vectors must not propagate
/// NaNs into probes; this clamps to `-Z` instead, mirroring runtimes.
#[inline]
#[must_use]
fn normalize_direction(direction: Vec3) -> Vec3 {
    if direction.length_squared() <= 1.0e-12 {
        return Vec3::NEG_Z;
    }
    direction.normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_normalizes_fractional_directions() {
        let probe = XrHitTestProbe::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -5.0));
        assert!((probe.direction.z + 1.0).abs() < 1e-5);

        let degenerate = XrHitTestProbe::new(Vec3::ZERO, Vec3::ZERO);
        assert_eq!(degenerate.direction, Vec3::NEG_Z);
    }

    #[test]
    fn resolve_sorts_and_filters() {
        let probe = XrHitTestProbe::new(Vec3::ZERO, Vec3::NEG_Z).with_surfaces(true, false, false);
        let candidates = [
            XrHit::new(3.0, Vec3::Y, XrHitSurface::Mesh),
            XrHit::new(1.0, Vec3::Y, XrHitSurface::Plane),
            XrHit::new(1.0, Vec3::Y, XrHitSurface::Anchor),
            XrHit::new(-1.0, Vec3::Y, XrHitSurface::Plane),
        ];
        let hits = probe.resolve(&candidates);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].surface, XrHitSurface::Plane);
        assert!((hits[0].distance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn resolve_breaks_ties_by_surface_rank() {
        let probe = XrHitTestProbe::new(Vec3::ZERO, Vec3::NEG_Z);
        let candidates = [
            XrHit::new(1.0, Vec3::Y, XrHitSurface::Mesh),
            XrHit::new(1.0, Vec3::Y, XrHitSurface::Anchor),
            XrHit::new(1.0, Vec3::Y, XrHitSurface::Plane),
        ];
        let hits = probe.resolve(&candidates);
        assert_eq!(hits[0].surface, XrHitSurface::Anchor);
        assert_eq!(hits[1].surface, XrHitSurface::Plane);
        assert_eq!(hits[2].surface, XrHitSurface::Mesh);
    }

    #[test]
    fn hit_positions_follow_probe() {
        let probe = XrHitTestProbe::new(Vec3::new(0.0, 1.0, 0.0), Vec3::NEG_Z);
        let hit = XrHit::new(2.0, Vec3::Y, XrHitSurface::Mesh);
        let position = hit.position(&probe);
        assert!((position.z + 2.0).abs() < 1e-5);
        assert!((position.y - 1.0).abs() < 1e-5);
    }
}
