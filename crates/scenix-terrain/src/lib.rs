#![cfg_attr(not(feature = "std"), no_std)]

//! CPU-side terrain systems for scenix.
//!
//! This crate is the `v1.8.0` terrain layer: seeded heightmaps, chunked
//! LOD mesh generation, splat weights on the vertex/texture channels,
//! deterministic chunk streaming, and heightfield collision probing. It is
//! **GPU-free** — chunk registration and the splat draw path live behind
//! `scenix-renderer`'s `environment` feature, and the material state lives
//! in `scenix-material`'s `TerrainMaterial`.
//!
//! # Quick start
//!
//! ```rust
//! use scenix_terrain::{Heightmap, LodPolicy, TerrainConfig, TerrainGrid};
//!
//! let map = Heightmap::from_fn(33, 33, 1.0, |ix, iz| ((ix + iz) * 4) as f32);
//! let config = TerrainConfig::new(4, 16, LodPolicy::default_three_band());
//! let grid = TerrainGrid::generate(&map, config);
//! let geometry = grid.chunk_geometry(0, 0).expect("chunk geometry");
//! assert!(geometry.vertex_count() > 0);
//! ```
//!
//! # Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `std` (default) | Enables the std allocator path shared workspace-wide |
//! | `serde` | Serializes heightmaps, splat maps, and streaming state |
//!
//! # Renderer integration
//!
//! `TerrainGrid::chunk_geometry_with_splat` packs layer weights into vertex
//! colors `r/g/b = layer 3/2/1`, `a = layer 0` (or a splat texture via the
//! renderer slot). `register_terrain_material` (renderer) then draws chunks
//! through the splat WGGL pipeline blending `TerrainMaterial` layers. See
//! docs/api/scenix-terrain.md for the full guide.

extern crate alloc;

pub mod chunks;
pub mod heightmap;
pub mod lod;
pub mod noise;
pub mod splat;
pub mod streaming;

pub use chunks::{TerrainChunk, TerrainConfig, TerrainGrid};
pub use heightmap::{Heightmap, TerrainCollider};
pub use lod::{LodBand, LodPolicy};
pub use noise::{NoiseConfig, noise_heights};
pub use splat::{SPLAT_LAYER_COUNT, SplatLayer, SplatMap};
pub use streaming::{ChunkCoord, ChunkStreamer, StreamDiff};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_surface_routes_through_a_heightmap_pipeline() {
        let noise = NoiseConfig::new(11, 0.04).with_octaves(2);
        let map = Heightmap::from_fn(17, 17, 2.0, |ix, iz| noise.sample(ix as f32, iz as f32));
        let mut splat = SplatMap::default_flat(8, 8);
        splat.normalize_in_place();
        let grid = TerrainGrid::generate(
            &map,
            TerrainConfig::new(2, 8, LodPolicy::default_three_band()),
        );
        let chunk = &grid.chunks()[0];
        let geometry = grid
            .chunk_geometry(0, chunk.distance_from(1.0, 1.0) as u32 % 3)
            .expect("geometry");
        assert!(!geometry.positions.is_empty());
        assert_eq!(
            (chunk.chunk_x, chunk.chunk_z),
            (0, 0),
            "row-major grid order"
        );
        assert_eq!(SPLAT_LAYER_COUNT, 4);
    }
}
