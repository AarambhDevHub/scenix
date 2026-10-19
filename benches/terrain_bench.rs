//! Terrain generation benchmark (scenix v1.8.0).
//!
//! Profiles the three CPU-heavy terrain stages: noise heightmap fills,
//! chunked LOD geometry builds, and splat-weighted palette packing.

use std::hint::black_box;

use scenix_terrain::{Heightmap, LodPolicy, NoiseConfig, SplatLayer, TerrainConfig, TerrainGrid};

const SAMPLES: u32 = 24;

fn main() {
    let noise = NoiseConfig::new(2024, 0.03).with_octaves(4);
    let started = std::time::Instant::now();

    let mut vertex_total: u64 = 0;
    let mut sample_total: u64 = 0;
    for _ in 0..SAMPLES {
        let map = Heightmap::from_noise(129, 129, 2.0, &noise);
        let grid = TerrainGrid::generate(
            &map,
            TerrainConfig::new(4, 32, LodPolicy::default_three_band()),
        );
        let layers = SplatLayer::defaults();
        for chunk_index in 0..grid.chunks().len() {
            geometry_sized(&grid, chunk_index, black_box(chunk_index as u32 % 3));
        }
        vertex_total += grid
            .chunk_geometry(0, 0)
            .map_or(0, |g| g.vertex_count() as u64);
        sample_total += map.heights.len() as u64;
        black_box(layers[0].tiling);
    }

    let elapsed = started.elapsed().as_secs_f64();
    println!("terrain_bench: {SAMPLES} iterations in {elapsed:.4}s");
    println!("  sampled heights : {sample_total}");
    println!("  built vertices  : {vertex_total}");
    println!(
        "  μs per unit     : {:.3}",
        (elapsed * 1.0e6) / (SAMPLES as f64)
    );
}

fn geometry_sized(grid: &TerrainGrid, chunk: usize, level: u32) -> usize {
    let _ = chunk;
    grid.chunk_geometry_with_splat(0, level, &|_, _, height| {
        if height > 0.5 {
            Some([0.4, 0.4, 0.1, 0.1])
        } else {
            Some([1.0, 0.0, 0.0, 0.0])
        }
    })
    .map_or(0, |geometry| geometry.vertex_count())
}
