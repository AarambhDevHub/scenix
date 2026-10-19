use scenix::{
    ChunkStreamer, Heightmap, LodBand, LodPolicy, NoiseConfig, TerrainCollider, TerrainConfig,
    TerrainGrid, Vec3,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A deterministic heightfield: seeded value noise replaying identically.
    let noise = NoiseConfig::new(2026, 0.03).with_octaves(4).with_gain(0.45);
    let map = Heightmap::from_noise(65, 65, 1.0, &noise);
    let replay = Heightmap::from_noise(65, 65, 1.0, &noise);
    assert_eq!(map.heights, replay.heights, "noise must replay");

    // Four-by-four chunks of 16 cells, with a two-band LOD policy.
    let policy =
        LodPolicy::new(&[LodBand::new(40.0, 0), LodBand::new(140.0, 1)]).expect("valid policy");
    let grid = TerrainGrid::generate(&map, TerrainConfig::new(4, 16, policy));
    println!(
        "grid: {} chunks over {} cells",
        grid.chunks().len(),
        map.width * map.depth
    );

    // Per-chunk geometry shrinks with distance without losing the surface.
    let near = grid.chunk_geometry(0, 0).expect("near chunk geometry");
    let far = grid.chunk_geometry(0, 1).expect("far chunk geometry");
    println!(
        "chunk 0 near level 0: {} verts / far level 1: {} verts",
        near.vertex_count(),
        far.vertex_count()
    );

    // Vertex heights always round-trip to the heightmap sample they came
    // from, regardless of LOD resampling.
    for vertex in near.positions.iter().take(4) {
        let expected = map.sample_bilinear(vertex.x, vertex.z);
        assert!((vertex.y - expected).abs() < 1.0e-4);
    }

    // Chunk streaming walks a camera around the lattice (5×5 window).
    let mut streamer = ChunkStreamer::new(2);
    let center = (2_u32, 2_u32);
    let lattice = (8_u32, 8_u32);
    let first = streamer.poll(lattice.0, lattice.1, center);
    println!("initial stream loads {} chunks", first.load.len());
    let idle = streamer.poll(lattice.0, lattice.1, center);
    assert!(idle.is_empty(), "steady positions stream nothing");
    let moved = streamer.poll(lattice.0, lattice.1, (3, 2));
    println!(
        "one-column move: {} loads / {} unloads",
        moved.load.len(),
        moved.unload.len()
    );

    // The collider shoots rays against the heightfield. Remap noise into a
    // positive range so every drop ray from 64 m lands inside the budget.
    let mut map = map;
    let (raw_low, raw_high) = map.remap_range(2.0, 14.0);
    let (min, max) = map.height_range();
    let collider = TerrainCollider::new(map).with_step(0.5).with_max_steps(256);
    let down = scenix::Ray3 {
        origin: Vec3::new(32.0, 64.0, 32.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
    };
    let distance = collider.probe(&down).expect("ray enters terrain");
    println!(
        "height range {min:.2}..{max:.2} (raw {raw_low:.2}..{raw_high:.2}); collider hit {distance:.2} m down"
    );

    Ok(())
}
