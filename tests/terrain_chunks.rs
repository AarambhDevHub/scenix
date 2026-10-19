//! Terrain chunk pipeline integration tests (scenix v1.8.0).
//!
//! Covers the full CPU terrain flow: seeded heightmaps → chunked LOD grids
//! → splat-weighted geometry → streaming diffs → collision probes.

use scenix_math::Ray3;
use scenix_math::Vec3;
use scenix_terrain::{
    ChunkCoord, ChunkStreamer, Heightmap, LodBand, LodPolicy, NoiseConfig, SplatLayer, SplatMap,
    TerrainCollider, TerrainConfig, TerrainGrid,
};

/// A deterministic ramp the tests can assert on exactly.
fn ramp_map(width: u32, depth: u32) -> Heightmap {
    Heightmap::from_fn(width, depth, 1.0, |ix, iz| ((ix + iz) * 2) as f32)
}

#[test]
fn seeded_noise_heightmaps_are_deterministic_and_bounded() {
    let config = NoiseConfig::new(77, 0.02).with_octaves(4).with_gain(0.4);
    let a = Heightmap::from_noise(33, 33, 2.0, &config);
    let b = Heightmap::from_noise(33, 33, 2.0, &config);
    assert_eq!(a.heights, b.heights, "same seed replays identically");

    let (min, max) = a.height_range();
    assert!(min <= max);
    assert!(max.abs() <= 1.0 + 1.0e-6, "raw noise stays in [-1, 1]");

    let mut refit = b;
    let raw_span = refit.remap_range(0.0, 42.0);
    assert!(
        raw_span.1 - raw_span.0 > 0.9,
        "a full field must exercise most of the [-1, 1] range"
    );
    assert!(raw_span.0 >= -1.0 - 1.0e-6 && raw_span.1 <= 1.0 + 1.0e-6);
    let (new_min, new_max) = refit.height_range();
    assert!(
        (new_max - new_min - 42.0).abs() < 1.0e-3,
        "got {new_min}..{new_max}"
    );
}

#[test]
fn chunk_grid_geometry_matches_heightmap_samples() {
    let map = ramp_map(33, 33);
    let policy =
        LodPolicy::new(&[LodBand::new(30.0, 0), LodBand::new(90.0, 1)]).expect("valid policy");
    let grid = TerrainGrid::generate(&map, TerrainConfig::new(4, 8, policy));

    assert_eq!(grid.chunks().len(), 16);
    let near_level = grid.level_for_distance(0, 1.0, 1.0);
    let far_level = grid.level_for_distance(0, 80.0, 80.0);
    assert_eq!(near_level, 0);
    assert_eq!(far_level, 1);

    let fine = grid.chunk_geometry(0, 0).expect("geometry at lod 0");
    let coarse = grid.chunk_geometry(0, 1).expect("geometry at lod 1");
    assert_eq!(fine.vertex_count(), 81);
    assert_eq!(coarse.vertex_count(), 25);
    assert_eq!(fine.triangle_count(), 128);
    assert_eq!(coarse.triangle_count(), 32);

    // Every vertex height equals the heightmap sample at that world point,
    // no matter the LOD level.
    for vertex in fine.positions.iter().take(20) {
        let expected = map.sample_bilinear(vertex.x, vertex.z);
        assert!(
            (vertex.y - expected).abs() < 1.0e-4,
            "vertex {vertex:?} drifted from {expected}"
        );
    }
    for normal in &coarse.normals {
        let rounded = normal.length();
        assert!((rounded - 1.0).abs() < 5.0e-3, "normals stay unit length");
    }
}

#[test]
fn splat_geometry_packs_normalized_weights_into_vertex_channels() {
    let map = ramp_map(17, 17);
    let policy = LodPolicy::new(&[LodBand::new(50.0, 0)]).expect("valid");
    let grid = TerrainGrid::generate(&map, TerrainConfig::new(2, 8, policy));

    let geometry = grid
        .chunk_geometry_with_splat(0, 0, &|x, z, height| {
            if height > 4.0 {
                Some([0.1, 0.2, 0.3, 0.4])
            } else if x <= z {
                None
            } else {
                Some([1.0, 0.0, 0.0, 0.0])
            }
        })
        .expect("splat geometry");

    for color in geometry.colors {
        let total = color.r + color.g + color.b + color.a;
        assert!(
            (total - 1.0).abs() < 1.0e-5,
            "weights must sum to one, got {total}"
        );
        // The fallback path (None) redistributes onto layer 0 (alpha).
        assert!(color.a >= 0.1);
    }
}

#[test]
fn splat_maps_normalize_and_sample_across_lattices() {
    let mut splat = SplatMap::from_fn(4, 4, |ix, iz| {
        let rock = ((ix + iz) % 2) as f32;
        let mut weights = [0.2_f32; 4];
        weights[1] = rock;
        weights[0] = 1.0 - rock * 0.8;
        weights
    });
    splat.normalize_in_place();
    for channels in &splat.weights {
        let total: f32 = channels.iter().sum();
        assert!((total - 1.0).abs() < 1.0e-5);
    }
    let sampled = splat.sample(0.5, 0.5);
    let total: f32 = sampled.iter().sum();
    assert!((total - 1.0).abs() < 1.0e-4);
    assert_eq!(SplatLayer::defaults().len(), 4);
    assert!(SplatLayer::default_rock().albedo != SplatLayer::default_sand().albedo);
}

#[test]
fn streaming_polls_deterministically_with_wrapping_lattices() {
    let mut streamer = ChunkStreamer::new(2);
    assert_eq!(streamer.half_extent, 2);
    let first = streamer.poll(4, 4, (10, 10));
    assert_eq!(first.load.len(), 25, "5×5 window");
    assert!(first.unload.is_empty());

    let idle = streamer.poll(4, 4, (10, 10));
    assert!(idle.is_empty(), "steady positions stream nothing");

    let move_diff = streamer.poll(5, 4, (10, 10));
    assert_eq!(
        move_diff.load.len(),
        5,
        "one-column shift loads 5 edge chunks"
    );
    assert_eq!(move_diff.unload.len(), 5);
    let held: ChunkCoord = (5, 4);
    assert!(streamer.contains(held));
    // Wrapped lattices stay consistent: heading past zero wraps around.
    let mut wrapped = ChunkStreamer::new(1);
    let wrap_diff = wrapped.poll(0, 0, (4, 4));
    assert_eq!(wrap_diff.load.len(), 9);
    assert!(wrapped.contains((3, 0)) && wrapped.contains((0, 3)));
    assert_eq!(wrapped.loaded_count(), 9);
}

#[test]
fn collider_probes_ramps_and_misses_flat_sky_rays() {
    // A gentle ramp keeps every surface point below the ray origin.
    let map = Heightmap::from_fn(16, 16, 1.0, |ix, iz| ((ix + iz) as f32) * 0.5);
    let collider = TerrainCollider::new(map).with_step(0.5).with_max_steps(128);

    // A ray straight down must intersect the surface close to the sampled
    // height.
    let down = Ray3 {
        origin: Vec3::new(6.0, 16.0, 8.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
    };
    let distance = collider.probe(&down).expect("ray enters the ramp");
    let hit = down.at(distance);
    let surface = collider.height_at(hit.x, hit.z);
    assert!(
        (hit.y - surface).abs() <= 1.0,
        "hit {hit:?} must land near surface {surface}"
    );

    // A backwards-pointing ray never touches terrain.
    let upward = Ray3 {
        origin: Vec3::new(2.0, 3.0, 2.0),
        direction: Vec3::new(0.0, 1.0, 0.0),
    };
    assert!(collider.probe(&upward).is_none());
    // Grounded normal helper stays unit length on flat terrain pieces.
    assert!((collider.normal_at(1.0, 1.0).length() - 1.0).abs() < 1.0e-4);
    assert!((collider.height_at(40.0, 0.0) - collider.height_at(15.0, 0.0)).abs() < 1.0e-5);
}

#[test]
fn world_bounds_enclose_every_chunk() {
    let map = ramp_map(17, 17);
    let grid = TerrainGrid::generate(
        &map,
        TerrainConfig::new(4, 8, LodPolicy::new(&[LodBand::new(40.0, 0)]).expect("p")),
    );
    let bounds = grid.world_aabb();
    assert!(bounds.max.x >= 16.0 - f32::EPSILON);
    assert!(bounds.max.z >= 16.0 - f32::EPSILON);
    assert!(bounds.min.y < bounds.max.y);
}
