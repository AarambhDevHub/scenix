//! Height grids, bilinear sampling, slope normals, heightfield collision
//! probing, and the seeded generator helpers used by example terrain.

use alloc::vec::Vec;

use scenix_math::{Ray3, Vec3};

use crate::NoiseConfig;

/// A rectangular height grid over a uniform XZ lattice.
///
/// Heights are meters, row-major in `[depth][width]` order; world position of
/// lattice cell `(ix, iz)` is `(ix * cell_size, h, iz * cell_size)`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Heightmap {
    /// Lattice width in cells.
    pub width: u32,
    /// Lattice depth in cells.
    pub depth: u32,
    /// Meters per lattice cell.
    pub cell_size: f32,
    /// Row-major heights.
    pub heights: Vec<f32>,
}

impl Heightmap {
    /// Builds a height grid from per-cell heights returned by `fill`.
    ///
    /// `fill` receives `(ix, iz)`; zero dimensions yield an empty map.
    #[must_use]
    pub fn from_fn(width: u32, depth: u32, cell_size: f32, fill: impl Fn(u32, u32) -> f32) -> Self {
        let mut heights = Vec::with_capacity((width * depth) as usize);
        for iz in 0..depth {
            for ix in 0..width {
                heights.push(fill(ix, iz));
            }
        }
        Self {
            width,
            depth,
            cell_size: if cell_size > 0.0 { cell_size } else { 1.0 },
            heights,
        }
    }

    /// Builds a height grid filled by stateless value noise.
    #[must_use]
    pub fn from_noise(width: u32, depth: u32, cell_size: f32, config: &NoiseConfig) -> Self {
        let mut heights = Vec::with_capacity((width * depth) as usize);
        for iz in 0..depth {
            for ix in 0..width {
                heights.push(config.sample(
                    ix as f32 * Self::cell(cell_size),
                    iz as f32 * Self::cell(cell_size),
                ));
            }
        }
        Self {
            width,
            depth,
            cell_size: Self::cell(cell_size),
            heights,
        }
    }

    /// Clamps a non-positive cell size to one meter.
    #[inline]
    const fn cell(cell_size: f32) -> f32 {
        if cell_size > 0.0 { cell_size } else { 1.0 }
    }

    /// World-space footprint along X.
    #[inline]
    pub const fn world_width(&self) -> f32 {
        (self.width as f32 - 1.0) * self.cell_size
    }

    /// World-space footprint along Z.
    #[inline]
    pub const fn world_depth(&self) -> f32 {
        (self.depth as f32 - 1.0) * self.cell_size
    }

    /// Lattice height at `(ix, iz)`, falling back to `0.0` off-grid.
    #[inline]
    pub fn height(&self, ix: u32, iz: u32) -> f32 {
        if ix >= self.width || iz >= self.depth {
            return 0.0;
        }
        self.heights[(iz * self.width + ix) as usize]
    }

    /// Overwrites one lattice height, clamped into the lattice.
    #[inline]
    pub fn set_height(&mut self, ix: u32, iz: u32, height: f32) {
        if ix < self.width && iz < self.depth {
            self.heights[(iz * self.width + ix) as usize] = height;
        }
    }

    /// Bilinear interpolation across the lattice; outside coordinates clamp
    /// onto the border cells.
    #[must_use]
    pub fn sample_bilinear(&self, world_x: f32, world_z: f32) -> f32 {
        if self.width == 0 || self.depth == 0 {
            return 0.0;
        }
        let cell = self.cell_size.max(1.0e-6);
        let fx = (world_x / cell).clamp(0.0, self.width as f32 - 1.0);
        let fz = (world_z / cell).clamp(0.0, self.depth as f32 - 1.0);
        // fx/fz are non-negative, so truncation equals floor.
        let ix = fx as u32;
        let iz = fz as u32;
        let ix1 = (ix + 1).min(self.width - 1);
        let iz1 = (iz + 1).min(self.depth - 1);
        let tx = fx - ix as f32;
        let tz = fz - iz as f32;
        let top = self.height(ix, iz) + (self.height(ix1, iz) - self.height(ix, iz)) * tx;
        let bottom = self.height(ix, iz1) + (self.height(ix1, iz1) - self.height(ix, iz1)) * tx;
        top + (bottom - top) * tz
    }

    /// Central-difference terrain normal at a world position.
    #[must_use]
    #[inline]
    pub fn normal_at(&self, world_x: f32, world_z: f32) -> Vec3 {
        let step = self.cell_size.max(1.0e-6);
        let left = self.sample_bilinear(world_x - step, world_z);
        let right = self.sample_bilinear(world_x + step, world_z);
        let front = self.sample_bilinear(world_x, world_z - step);
        let back = self.sample_bilinear(world_x, world_z + step);
        let slope = Vec3::new(-(right - left), 2.0 * step, -(back - front));
        slope.normalize()
    }

    /// Minimum and maximum heights, `(min, max)`; an empty map returns zeros.
    #[must_use]
    pub fn height_range(&self) -> (f32, f32) {
        let (mut min, mut max) = (f32::INFINITY, f32::NEG_INFINITY);
        for height in &self.heights {
            min = min.min(*height);
            max = max.max(*height);
        }
        if min > max { (0.0, 0.0) } else { (min, max) }
    }

    /// Axis-aligned world-space bounds of the sampled terrain surface.
    #[must_use]
    pub fn world_bounds(&self) -> scenix_math::Aabb {
        let (min, max) = self.height_range();
        scenix_math::Aabb {
            min: scenix_math::Vec3::new(0.0, min, 0.0),
            max: scenix_math::Vec3::new(
                self.world_width().max(0.0),
                max,
                self.world_depth().max(0.0),
            ),
        }
    }

    /// Rescales every height so `(min, max)` becomes `(low, high)`.
    ///
    /// Degenerate maps are clamped to `mid`. Returns `(old_min, old_max)`.
    pub fn remap_range(&mut self, low: f32, high: f32) -> (f32, f32) {
        if self.heights.is_empty() {
            return (0.0, 0.0);
        }
        let mut low_target = low;
        let mut high_target = high;
        if low_target > high_target {
            core::mem::swap(&mut low_target, &mut high_target);
        }
        let (min, max) = self.height_range();
        let span = max - min;
        if span <= 1.0e-12 {
            self.heights.fill((low_target + high_target) * 0.5);
            return (min, max);
        }
        let target_span = high_target - low_target;
        for height in &mut self.heights {
            *height = low_target + ((*height - min) / span) * target_span;
        }
        (min, max)
    }
}

/// Heightfield collision queries over a [`Heightmap`].
///
/// The probe marches the ray in uniform steps and treats the front where the
/// ray first drops under the terrain surface as the hit; the returned distance
/// is refined by a quarter-step backtrack, which keeps deterministic
/// correctness while remaining cheap in CPU game loops.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TerrainCollider {
    map: Heightmap,
    /// Ray marching step in meters.
    pub step: f32,
    /// How many steps a probe may march before giving up.
    pub max_steps: u32,
}

impl TerrainCollider {
    /// Wraps a heightmap with the default 0.5 m step and 512-step cap.
    #[inline]
    pub const fn new(map: Heightmap) -> Self {
        Self {
            map,
            step: 0.5,
            max_steps: 512,
        }
    }

    /// Returns this collider with a custom walk step.
    #[inline]
    pub const fn with_step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Returns this collider with a custom step budget.
    #[inline]
    pub const fn with_max_steps(mut self, max_steps: u32) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// The wrapped height grid.
    #[inline]
    pub const fn map(&self) -> &Heightmap {
        &self.map
    }

    /// Takes ownership of the wrapped height grid.
    #[inline]
    pub fn into_map(self) -> Heightmap {
        self.map
    }

    /// Terrain height under `(x, z)`.
    #[inline]
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        self.map.sample_bilinear(x, z)
    }

    /// Terrain slope normal under `(x, z)`.
    #[inline]
    pub fn normal_at(&self, x: f32, z: f32) -> Vec3 {
        self.map.normal_at(x, z)
    }

    /// Marches `ray` and returns the distance where it enters the terrain,
    /// or `None` when the ray stays above the surface for every step.
    #[must_use]
    pub fn probe(&self, ray: &Ray3) -> Option<f32> {
        let step = self.step.max(1.0e-3);
        let mut index = 0;
        while index < self.max_steps {
            let t = index as f32 * step;
            let at = ray.at(t);
            if at.y > 65_536.0 && ray.direction.y >= 0.0 {
                return None;
            }
            let surface = self.height_at(at.x, at.z);
            if at.y <= surface {
                let back = (t - step * 0.25).max(0.0);
                return Some(back);
            }
            index += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use scenix_math::{Ray3, Vec3};

    use super::{Heightmap, TerrainCollider};

    #[test]
    fn ramp_samples_interpolate_between_lattice_corners() {
        // Height ramps linearly with z: cell rows.
        let map = Heightmap::from_fn(4, 4, 1.0, |ix, iz| (ix + iz) as f32);
        assert_eq!(map.height(0, 0), 0.0);
        assert_eq!(map.height(3, 3), 6.0);
        let mid = map.sample_bilinear(0.5, 0.5);
        assert!((mid - 1.0).abs() <= 1.0e-6);
        assert_eq!(map.world_width(), 3.0);
        let (min, max) = map.height_range();
        assert_eq!((min, max), (0.0, 6.0));
    }

    #[test]
    fn flat_maps_point_normals_straight_up() {
        let map = Heightmap::from_fn(8, 8, 1.0, |_, _| 3.0);
        let normal = map.normal_at(3.5, 3.5);
        assert!((normal.y - 1.0).abs() < 1.0e-5);
        assert!(normal.x.abs() < 1.0e-6 && normal.z.abs() < 1.0e-6);
    }

    #[test]
    fn remap_lifts_and_spans_the_requested_range() {
        let mut map = Heightmap::from_fn(4, 4, 1.0, |ix, iz| ((ix + iz) * 2) as f32);
        let (old_min, old_max) = map.remap_range(-1.0, 1.0);
        assert_eq!(old_min, 0.0);
        assert_eq!(old_max, 12.0);
        let (min, max) = map.height_range();
        assert!(min >= -1.0 - 1.0e-5 && max <= 1.0 + 1.0e-5);
        assert!((max - min - 2.0).abs() < 1.0e-4);
    }

    #[test]
    fn collider_probe_hits_a_ramp_surface() {
        let map = Heightmap::from_fn(8, 8, 1.0, |_, iz| iz as f32);
        let collider = TerrainCollider::new(map).with_step(0.25).with_max_steps(64);
        let ray = Ray3 {
            origin: Vec3::new(2.0, 6.0, 0.0),
            direction: Vec3::new(0.0, -1.0, 0.0).normalize(),
        };
        let Some(distance) = collider.probe(&ray) else {
            panic!("ray must enter terrain");
        };
        let hit = ray.at(distance);
        let surface = collider.height_at(hit.x, hit.z);
        assert!(
            (hit.y - surface).abs() <= 1.0,
            "hit {hit:?} surfaced {surface}"
        );
    }

    #[test]
    fn collider_misses_are_none_and_off_grid_height_is_zero() {
        let map = Heightmap::from_fn(4, 4, 1.0, |_, _| 0.0);
        let collider = TerrainCollider::new(map).with_step(0.5).with_max_steps(4);
        let ray = Ray3 {
            origin: Vec3::new(0.0, 2.0, 0.0),
            direction: Vec3::new(1.0, 0.0, 0.0).normalize(),
        };
        assert!(collider.probe(&ray).is_none());
        assert_eq!(collider.height_at(-50.0, 3.0), 0.0);
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::Heightmap;

    #[test]
    fn serde_round_trips_the_grid() {
        let map = Heightmap::from_fn(2, 2, 1.0, |ix, iz| (ix + iz) as f32);
        let json = serde_json::to_string(&map).expect("serde ok");
        let back: Heightmap = serde_json::from_str(&json).expect("serde back");
        assert_eq!(back.heights, map.heights);
        assert_eq!(back.cell_size, 1.0);
    }
}
