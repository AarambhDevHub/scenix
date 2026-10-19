//! Chunked terrain mesh generation with per-chunk LOD geometry.
//!
//! The world is split into a `chunks × chunks` grid; every chunk maps to a
//! [`scenix_mesh::Geometry`] whose resolution follows a [`LodPolicy`] level.
//! Splat weights ride the vertex-color channel `r/g/b = layer 1/2/3` with
//! `a = layer 0` when `TerrainGrid::generate_with_splat` is used, matching
//! the scenix terrain WGSL convention.

use alloc::vec::Vec;

use scenix_math::{Aabb, Vec2, Vec3};
use scenix_mesh::Geometry;

use scenix_math::float::sqrt;

use crate::heightmap::Heightmap;
use crate::lod::LodPolicy;

/// Geometry settings for one generated terrain world.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TerrainConfig {
    /// Chunks along each axis (square chunk count).
    pub chunk_count: u32,
    /// Base vertex resolution per chunk axis at LOD 0; clamped to `2..=256`.
    pub base_resolution: u32,
    /// LOD policy used when callers resolve per-chunk levels.
    pub lod: LodPolicy,
}

impl TerrainConfig {
    /// Creates a chunked configuration; invalid knobs clamp to safe values.
    #[inline]
    pub fn new(chunk_count: u32, base_resolution: u32, lod: LodPolicy) -> Self {
        Self {
            chunk_count: chunk_count.max(1),
            base_resolution: base_resolution.clamp(2, 256),
            lod,
        }
    }
}

/// One registered terrain cell.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TerrainChunk {
    /// Grid column of this chunk (0 based).
    pub chunk_x: u32,
    /// Grid row of this chunk (0 based).
    pub chunk_z: u32,
    /// World-space min corner of the chunk in meters.
    pub min_x: f32,
    /// World-space min depth edge of the chunk in meters.
    pub min_z: f32,
    /// Chunk footprint per axis in meters (equal on both axes).
    pub chunk_size: f32,
}

impl TerrainChunk {
    /// Distance from a world point to the chunk AABB (0 when inside).
    #[must_use]
    pub fn distance_from(&self, world_x: f32, world_z: f32) -> f32 {
        let max_x = self.min_x + self.chunk_size;
        let max_z = self.min_z + self.chunk_size;
        let dx = (self.min_x - world_x).max(world_x - max_x).max(0.0);
        let dz = (self.min_z - world_z).max(world_z - max_z).max(0.0);
        sqrt(dx * dx + dz * dz)
    }
}

/// The generated chunk list for one heightmap.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TerrainGrid {
    map: Heightmap,
    config: TerrainConfig,
    chunks: Vec<TerrainChunk>,
}

impl TerrainGrid {
    /// Splits `map` into `config.chunk_count²` chunks.
    ///
    /// The heightmap's world footprint must cover the chunk lattice: chunk
    /// size equals `world_width / chunk_count` along X, mirroring depth.
    #[must_use]
    pub fn generate(map: &Heightmap, config: TerrainConfig) -> Self {
        let span = (map.world_width() + map.world_depth()) * 0.5;
        let chunk_size = span / config.chunk_count as f32;
        let mut chunks = Vec::with_capacity((config.chunk_count * config.chunk_count) as usize);
        for cz in 0..config.chunk_count {
            for cx in 0..config.chunk_count {
                chunks.push(TerrainChunk {
                    chunk_x: cx,
                    chunk_z: cz,
                    min_x: cx as f32 * chunk_size,
                    min_z: cz as f32 * chunk_size,
                    chunk_size,
                });
            }
        }
        Self {
            map: map.clone(),
            config,
            chunks,
        }
    }

    /// The source heightmap snapshot.
    #[inline]
    pub const fn map(&self) -> &Heightmap {
        &self.map
    }

    /// The generation configuration.
    #[inline]
    pub const fn config(&self) -> &TerrainConfig {
        &self.config
    }

    /// The chunk descriptors in row-major order.
    #[inline]
    pub fn chunks(&self) -> &[TerrainChunk] {
        &self.chunks
    }

    /// Chunk count along one axis.
    #[inline]
    pub const fn chunk_count(&self) -> u32 {
        self.config.chunk_count
    }

    /// Alias-complete world AABB spanning every chunk.
    #[must_use]
    pub fn world_aabb(&self) -> Aabb {
        let raw = self.map.world_bounds();
        let chunk_size = self
            .chunks
            .first()
            .map_or(raw.max.x, |chunk| chunk.chunk_size);
        let count = self.check_count();
        Aabb {
            min: scenix_math::Vec3::new(0.0, raw.min.y, 0.0),
            max: scenix_math::Vec3::new(
                (chunk_size * count as f32).max(raw.max.x),
                raw.max.y,
                (chunk_size * count as f32).max(raw.max.z),
            ),
        }
    }

    /// Chunk count as usize for bounds math (`>= 1`).
    #[inline]
    const fn check_count(&self) -> u32 {
        if self.config.chunk_count == 0 {
            1
        } else {
            self.config.chunk_count
        }
    }

    /// LOD level picked by the grid policy for a chunk distance.
    #[must_use]
    pub fn level_for_distance(&self, chunk_index: usize, world_x: f32, world_z: f32) -> u32 {
        match self.chunks.get(chunk_index) {
            Some(chunk) => {
                let distance = chunk.distance_from(world_x, world_z);
                self.config.lod.select(distance)
            }
            None => 0,
        }
    }

    /// Generates one chunk at a LOD level; identical parameters always
    /// produce identical geometry.
    ///
    /// Vertices follow the world lattice; heights sample the map bilinearly
    /// so LOD levels never crack at seams more than a fraction of the map's
    /// own sampling error. UVs map the full world footprint, so splat
    /// textures chain across chunks seamlessly.
    #[must_use]
    pub fn chunk_geometry(&self, chunk_index: usize, level: u32) -> Option<Geometry> {
        let chunk = self.chunks.get(chunk_index)?;
        Some(self.build_chunk_geometry(chunk, level, &|_, _, _| None))
    }

    /// Same as [`TerrainGrid::chunk_geometry`] with a splat-weight callback.
    ///
    /// `splat` returns an optional normalized `[w0, w1, w2, w3]` per world
    /// `(x, z, height)`; a `None` return falls back to flat layer 0. Weights
    /// ride the vertex-color channel in the WGSL order `(w3, w2, w1, w0)`
    /// remapped to `(r, g, b, a)`.
    #[must_use]
    pub fn chunk_geometry_with_splat(
        &self,
        chunk_index: usize,
        level: u32,
        splat: &impl Fn(f32, f32, f32) -> Option<[f32; 4]>,
    ) -> Option<Geometry> {
        let chunk = self.chunks.get(chunk_index)?;
        Some(self.build_chunk_geometry(chunk, level, splat))
    }

    fn build_chunk_geometry(
        &self,
        chunk: &TerrainChunk,
        level: u32,
        splat: &impl Fn(f32, f32, f32) -> Option<[f32; 4]>,
    ) -> Geometry {
        let resolution = LodPolicy::level_resolution(self.config.base_resolution, level);
        let vertex_count = (resolution + 1) as usize;
        let step = chunk.chunk_size / resolution as f32;
        let world_width = self.map.world_width();
        let world_depth = self.map.world_depth();

        let mut positions = Vec::with_capacity(vertex_count * vertex_count);
        let mut normals = Vec::with_capacity(vertex_count * vertex_count);
        let mut uvs = Vec::with_capacity(vertex_count * vertex_count);
        let mut colors = Vec::with_capacity(vertex_count * vertex_count);

        for row in 0..=resolution {
            for column in 0..=resolution {
                let world_x = chunk.min_x + column as f32 * step;
                let world_z = chunk.min_z + row as f32 * step;
                let height = self.map.sample_bilinear(world_x, world_z);
                positions.push(Vec3::new(world_x, height, world_z));
                normals.push(self.map.normal_at(world_x, world_z));
                let u = world_x / world_width.max(1.0e-6);
                let v = world_z / world_depth.max(1.0e-6);
                uvs.push(Vec2::new(u, v));
                match splat(world_x, world_z, height) {
                    Some(weights) => colors.push(splat_color(&weights)),
                    None => colors.push(scenix_core::Color::rgba(0.0, 0.0, 0.0, 1.0)),
                }
            }
        }

        let mut indices = Vec::with_capacity((resolution * resolution * 6) as usize);
        for row in 0..resolution {
            for column in 0..resolution {
                let top_left = row * (resolution + 1) + column;
                let top_right = top_left + 1;
                let bottom_left = top_left + resolution + 1;
                let bottom_right = bottom_left + 1;
                indices.extend_from_slice(&[
                    top_left,
                    bottom_left,
                    top_right,
                    top_right,
                    bottom_left,
                    bottom_right,
                ]);
            }
        }

        Geometry {
            positions,
            normals,
            uvs,
            uvs2: Vec::new(),
            colors,
            indices,
            tangents: Vec::new(),
        }
    }
}

/// Splat weights `{ (w3, w2, w1, w0) -> (r, g, b, a) }` packed for the GPU.
#[inline]
fn splat_color(weights: &[f32; 4]) -> scenix_core::Color {
    scenix_core::Color::rgba(weights[3], weights[2], weights[1], weights[0])
}

#[cfg(test)]
mod tests {
    use scenix_mesh::Geometry;

    use crate::lod::{LodBand, LodPolicy};

    use super::{Heightmap, TerrainChunk, TerrainConfig, TerrainGrid};

    fn ramp_map() -> Heightmap {
        Heightmap::from_fn(17, 17, 1.0, |_, iz| iz as f32)
    }

    #[test]
    fn grid_populates_every_chunk_in_row_major_order() {
        let grid = TerrainGrid::generate(
            &ramp_map(),
            TerrainConfig::new(4, 8, LodPolicy::default_three_band()),
        );
        assert_eq!(grid.chunks().len(), 16);
        assert_eq!(grid.chunk_count(), 4);
        let first = &grid.chunks()[0];
        assert_eq!((first.chunk_x, first.chunk_z), (0, 0));
        let last = &grid.chunks()[15];
        assert_eq!((last.chunk_x, last.chunk_z), (3, 3));
        assert!(last.min_x > 0.0 && last.min_z > 0.0);

        let far = TerrainChunk {
            chunk_x: 0,
            chunk_z: 0,
            min_x: 0.0,
            min_z: 0.0,
            chunk_size: 4.0,
        };
        assert!(far.distance_from(100.0, 100.0) > 0.0);
        assert_eq!(far.distance_from(1.0, 1.0), 0.0);
    }

    #[test]
    fn chunk_geometry_tracks_lod_and_world_positions() {
        let grid = TerrainGrid::generate(
            &ramp_map(),
            TerrainConfig::new(2, 16, LodPolicy::default_three_band()),
        );
        let coarse = grid.chunk_geometry(0, 2).expect("chunk exists");
        let fine = grid.chunk_geometry(0, 0).expect("chunk exists");
        let coarse_count = coarse.vertex_count();
        let fine_count = fine.vertex_count();
        assert_eq!(coarse_count, 25, "lod 2 halves 16 twice => 4 + border");
        assert_eq!(fine_count, 289);
        assert!(coarse_count < fine_count);

        let first = fine.positions.first().expect("positions");
        assert!((first.y - grid.map().sample_bilinear(first.x, first.z)).abs() < 1.0e-4);

        let triangles = fine.triangle_count();
        assert!(triangles > 0);
        let extremes = fine
            .positions
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), p| {
                (min.min(p.y), max.max(p.y))
            });
        let height = grid.map().height_range();
        assert!(extremes.0 >= height.0 - 0.5 && extremes.1 <= height.1 + 0.5);
    }

    #[test]
    fn level_for_distance_follows_the_policy() {
        let grid = TerrainGrid::generate(
            &ramp_map(),
            TerrainConfig::new(4, 8, LodPolicy::default_three_band()),
        );
        assert_eq!(grid.level_for_distance(0, 0.0, 0.0), 0);
        assert_eq!(grid.level_for_distance(0, 200.0, 200.0), 2);
    }

    #[test]
    fn splat_weights_pack_into_vertex_colors_normalized() {
        let grid = TerrainGrid::generate(
            &ramp_map(),
            TerrainConfig::new(
                1,
                8,
                LodPolicy::new(&[LodBand::new(10.0, 0)]).expect("policy"),
            ),
        );
        let geometry: Geometry = grid
            .chunk_geometry_with_splat(0, 0, &|x, z, _| {
                if x > z {
                    Some([0.2, 0.3, 0.25, 0.25])
                } else {
                    None
                }
            })
            .expect("geometry");
        let covered = geometry
            .colors
            .iter()
            .filter(|color| (color.r + color.g + color.b + color.a - 1.0).abs() < 1.0e-5)
            .count();
        assert!(covered > 0, "some vertices carry a normalized weight set");
    }
}
