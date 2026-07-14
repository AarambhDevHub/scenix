use alloc::vec::Vec;

use scenix_core::Color;
use scenix_math::{Aabb, Vec3};

use crate::{BoundingBoxHelper, GridHelper, LineGeometry};

/// Selection outline helper for one or more world-space bounds.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectionHelper {
    /// Selected bounds.
    pub bounds: Vec<Aabb>,
    /// Outline color.
    pub color: Color,
}

impl SelectionHelper {
    /// Writes selection outlines into reusable storage.
    pub fn write_geometry(&self, geometry: &mut LineGeometry) {
        geometry.clear();
        geometry.reserve(self.bounds.len() * 24, 0);
        for bounds in &self.bounds {
            geometry.merge(&BoundingBoxHelper::new(*bounds, self.color).to_geometry());
        }
    }

    /// Generates owned selection geometry.
    pub fn to_geometry(&self) -> LineGeometry {
        let mut geometry = LineGeometry::new();
        self.write_geometry(&mut geometry);
        geometry
    }
}

/// Corner-emphasized editor bounds helper.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoundsGizmoHelper {
    /// Bounds to visualize.
    pub bounds: Aabb,
    /// Corner segment fraction of each edge.
    pub corner_fraction: f32,
    /// Line color.
    pub color: Color,
}

impl BoundsGizmoHelper {
    /// Writes corner brackets into reusable storage.
    pub fn write_geometry(&self, geometry: &mut LineGeometry) {
        geometry.clear();
        let full = BoundingBoxHelper::new(self.bounds, self.color).to_geometry();
        let fraction = self.corner_fraction.clamp(0.01, 0.5);
        geometry.reserve(full.positions.len() * 2, 0);
        for segment in full.positions.chunks_exact(2) {
            let a = segment[0];
            let b = segment[1];
            let delta = b - a;
            geometry.push_segment(a, a + delta * fraction, self.color);
            geometry.push_segment(b, b - delta * fraction, self.color);
        }
    }

    /// Generates owned geometry.
    pub fn to_geometry(&self) -> LineGeometry {
        let mut geometry = LineGeometry::new();
        self.write_geometry(&mut geometry);
        geometry
    }
}

/// Grid helper translated to an editor work-plane origin.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SnapGridHelper {
    /// Grid center.
    pub origin: Vec3,
    /// Grid size.
    pub size: f32,
    /// Number of subdivisions.
    pub divisions: u32,
    /// Major-axis color.
    pub major_color: Color,
    /// Regular-line color.
    pub minor_color: Color,
}

impl SnapGridHelper {
    /// Writes grid lines into reusable storage.
    pub fn write_geometry(&self, geometry: &mut LineGeometry) {
        let source = GridHelper::new(self.size, self.divisions)
            .colors(self.major_color, self.minor_color)
            .to_geometry();
        geometry.clear();
        geometry.reserve(source.positions.len(), source.indices.len());
        geometry
            .positions
            .extend(source.positions.iter().map(|point| *point + self.origin));
        geometry.colors.extend_from_slice(&source.colors);
        geometry.indices.extend_from_slice(&source.indices);
    }

    /// Generates owned grid geometry.
    pub fn to_geometry(&self) -> LineGeometry {
        let mut geometry = LineGeometry::new();
        self.write_geometry(&mut geometry);
        geometry
    }
}
