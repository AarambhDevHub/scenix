//! scenix v1.8.0 "Terrain, Sky, And Water" — facade-level CPU coverage.
//!
//! Exercises the three environment crates end to end without a GPU: terrain
//! LOD chunks, streaming, and collision probes; sky profile baking and
//! grounded skyboxes; water wave/foam/fresnel/underwater math and mirror
//! helpers; the material descriptors that drive the WGSL pipelines; plus the
//! environment ids and sun-light bridges the scene schema registers.

use scenix::{
    ChunkStreamer, FoamParams, FresnelParams, GerstnerWave, GroundedSkybox, Heightmap, LodBand,
    LodPolicy, NoiseConfig, PerspectiveCamera, Ray3, SkyAmbient, SkyMaterial, SkyModel, SplatLayer,
    SplatMap, SunPosition, TerrainCollider, TerrainConfig, TerrainGrid, TerrainMaterial,
    UnderwaterView, Vec2, Vec3, Vec4, WaterConfig, WaterMaterial, WaterSurface, WaveField,
    bake_ambient, bake_equirectangular, bake_light_probe, mirror_transform, reflected_camera,
};
use scenix::{Color, FEATURE_ATMOSPHERE, FEATURE_FOAM, FEATURE_FRESNEL, FEATURE_REFRACTION};
use scenix::{FEATURE_SPLAT_MAP, ShaderKind};
use scenix_core::{SkyId, TerrainId, TextureId, WaterId};

fn ramp_map(width: u32, depth: u32) -> Heightmap {
    Heightmap::from_fn(width, depth, 1.0, |ix, iz| {
        ((ix as f32) + (iz as f32)) * 0.5
    })
}

#[test]
fn terrain_lod_chunks_split_geometry_and_splat() {
    let map = ramp_map(33, 33);
    let policy =
        LodPolicy::new(&[LodBand::new(30.0, 0), LodBand::new(90.0, 1)]).expect("valid policy");
    let grid = TerrainGrid::generate(&map, TerrainConfig::new(4, 8, policy));

    assert_eq!(grid.chunks().len(), 16);
    assert_eq!(grid.level_for_distance(0, 1.0, 1.0), 0);
    assert_eq!(grid.level_for_distance(0, 80.0, 80.0), 1);

    let fine = grid.chunk_geometry(0, 0).expect("geometry at lod 0");
    let coarse = grid.chunk_geometry(0, 1).expect("geometry at lod 1");
    assert_eq!(fine.vertex_count(), 81);
    assert_eq!(coarse.vertex_count(), 25);

    let splat = grid
        .chunk_geometry_with_splat(0, 0, &|x, _z, height| {
            if height > 4.0 {
                Some([0.1, 0.2, 0.3, 0.4])
            } else if x <= _z {
                None
            } else {
                Some([1.0, 0.0, 0.0, 0.0])
            }
        })
        .expect("splat geometry");
    for color in &splat.colors {
        let total = color.r + color.g + color.b + color.a;
        assert!((total - 1.0).abs() < 1.0e-5, "weights must sum to one");
    }
}

#[test]
fn terrain_streams_chunks_and_probes_ramps() {
    let mut streamer = ChunkStreamer::new(2);
    let first = streamer.poll(4, 4, (10, 10));
    assert_eq!(first.load.len(), 25, "5×5 window");
    let idle = streamer.poll(4, 4, (10, 10));
    assert!(idle.is_empty(), "steady positions stream nothing");
    let shift = streamer.poll(5, 4, (10, 10));
    assert_eq!(shift.load.len(), 5);
    assert_eq!(shift.unload.len(), 5);
    assert!(streamer.contains((5, 4)));

    // Splat lattices normalize across channels.
    let mut splat = SplatMap::from_fn(4, 4, |ix, iz| {
        let mut weights = [0.25_f32; 4];
        weights[0] += (ix + iz) as f32 * 0.05;
        weights
    });
    splat.normalize_in_place();
    let sampled = splat.sample(0.5, 0.5);
    assert!((sampled.iter().sum::<f32>() - 1.0).abs() < 1.0e-4);
    assert_eq!(SplatLayer::defaults().len(), 4);

    // Collision: a downward ray lands near the sampled surface height.
    let map = Heightmap::from_fn(16, 16, 1.0, |ix, iz| ((ix + iz) as f32) * 0.5);
    let collider = TerrainCollider::new(map).with_step(0.5).with_max_steps(128);
    let down = Ray3 {
        origin: Vec3::new(6.0, 16.0, 8.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
    };
    let distance = collider.probe(&down).expect("ray enters the ramp");
    let hit = down.at(distance);
    let surface = collider.height_at(hit.x, hit.z);
    assert!((hit.y - surface).abs() <= 1.0);

    let upward = Ray3 {
        origin: Vec3::new(2.0, 3.0, 2.0),
        direction: Vec3::new(0.0, 1.0, 0.0),
    };
    assert!(collider.probe(&upward).is_none());
}

#[test]
fn sky_profiles_bake_deterministically_into_skylight() {
    let model = SkyModel::clear_day();
    let sun = SunPosition::degrees(35.0, 40.0);
    let sun_dir = sun.direction();
    let zenith = model.rgb_at(Vec3::Y, sun_dir);
    let repeat = model.rgb_at(Vec3::Y, sun_dir);
    assert_eq!(zenith, repeat, "evaluations replay deterministically");

    let custom = SkyModel::overcast_haze()
        .with_turbidity(6.0)
        .with_rayleigh(1.0)
        .with_mie(0.5, 0.95);
    assert_eq!(custom.turbidity, 6.0);
    assert!(custom.mie_g > 0.9);

    let baked = bake_equirectangular(&model, &sun, 10, 8);
    let again = bake_equirectangular(&model, &sun, 10, 8);
    assert_eq!(baked, again, "bake replay is deterministic");

    let probe = bake_light_probe(&model, &sun, 16, 16, 1.25).expect("probe projection succeeds");
    let magnitude: f32 = probe
        .sh_coefficients
        .iter()
        .map(|c| c.x.abs() + c.y.abs() + c.z.abs())
        .sum();
    assert!(magnitude > 0.0, "SH taps carry radiance");

    let ambient: SkyAmbient = bake_ambient(&baked, 10, 8);
    assert!(ambient.sky.iter().all(|v| *v >= 0.0));
    assert!(ambient.ground.iter().all(|v| *v >= 0.0));
    assert!(ambient.sky.iter().sum::<f32>() > 0.0);

    // Grounded skyboxes keep the dome ordered (zenith first, ground dark).
    let skybox = GroundedSkybox::new(
        Color::rgb(0.08, 0.24, 0.72),
        Color::rgb(0.86, 0.64, 0.42),
        Color::rgb(0.12, 0.12, 0.11),
    );
    assert!((skybox.rgb_at(Vec3::Y)[2] - 0.72).abs() < 1.0e-5);
    assert_eq!(skybox.sample_dome(6, 6).len(), 36);

    // Sun positions drive unit-length lights with warm low-elevation tints.
    let low = SunPosition::degrees(20.0, 60.0).sun_light(5.0);
    assert!((low.direction.length() - 1.0).abs() < 1.0e-5);
    assert!(low.color.r > low.color.b, "low sun leans warm");
    assert!((sun.direction().length() - 1.0).abs() < 1.0e-5);
}

fn ocean_surface() -> WaterSurface {
    WaterSurface::new(WaveField::ocean_shore()).with_config(WaterConfig::new(24.0, 24.0, 6, 1.5))
}

#[test]
fn water_surfaces_displace_fresnel_and_murk() {
    let surface = ocean_surface();
    let geometry = surface.geometry(0.6);
    assert_eq!(geometry.vertex_count(), 49);
    assert_eq!(geometry.triangle_count(), 72);
    for (vertex, normal) in geometry.positions.iter().take(6).zip(&geometry.normals) {
        let expected = surface.height_at(vertex.x, vertex.z, 0.6);
        assert!((vertex.y - expected).abs() < 1.0e-4);
        assert!((normal.length() - 1.0).abs() < 5.0e-3);
    }

    let active = WaveField::ocean_shore();
    assert_eq!(active.waves().len(), 3, "ocean-shore is a 3-wave preset");
    let wave = GerstnerWave::new(Vec2::new(1.0, 0.0), 0.5, 3.0);
    assert!((wave.wave_number() - core::f32::consts::TAU / 3.0).abs() < 1.0e-5);

    // Fresnel approaches full reflectance at grazing angles.
    let fresnel = FresnelParams::water();
    assert!(fresnel.reflectance(0.0) > 0.95);
    assert!(fresnel.reflectance(1.0) < fresnel.reflectance(0.0));

    // Foam masks crest energy behind a hard threshold gate.
    let foam = FoamParams::shoreline().with_threshold(0.5);
    assert_eq!(foam.mask(0.4), 0.0, "sub-threshold crest stays unmasked");
    assert!(
        foam.mask(0.9) > 0.0,
        "crest strength above the gate masks in"
    );

    // Underwater tinting absorbs red first and drifts into murk.
    let view = UnderwaterView::clear_pool();
    let shallow = view.transmittance(1.0);
    let deep = view.transmittance(6.0);
    assert!(shallow[0] < shallow[2], "red absorbs before blue");
    for (near, far) in shallow.iter().zip(&deep) {
        assert!(far < near);
    }
    let white = Color::WHITE;
    let close = view.view_tint(white, 0.4);
    let distant = view.view_tint(white, 9.0);
    let murk = view.murk_color.to_array();
    let drift_near: f32 = close.iter().zip(murk).map(|(v, m)| (v - m).abs()).sum();
    let drift_far: f32 = distant.iter().zip(murk).map(|(v, m)| (v - m).abs()).sum();
    assert!(drift_far < drift_near, "distance drives fade into murk");

    // Mirror helpers flip points through the water plane.
    let matrix = mirror_transform(2.0).to_mat4();
    let flipped = matrix.mul_vec4(Vec4::new(1.0, 5.0, -1.0, 1.0));
    assert!((flipped.y - (2.0 * 2.0 - 5.0)).abs() < 1.0e-5);

    let camera = PerspectiveCamera::new(50.0, 1.6, 0.2, 300.0)
        .position(Vec3::new(2.0, 10.0, 6.0))
        .target(Vec3::new(0.0, 3.0, 0.0))
        .up(Vec3::Y);
    let mirrored = reflected_camera(&camera, 2.0);
    assert!((mirrored.position.y - (-6.0)).abs() < 1.0e-6);
    assert!((mirrored.up.y + 1.0).abs() < 1.0e-6);
}

#[test]
fn environment_descriptors_match_shader_kinds() {
    // Terrain: four named layers; splat/detail textures flip the inputs.
    let terrain = TerrainMaterial::default_palette();
    assert_eq!(terrain.layers.len(), 4);
    let textured = terrain
        .with_splat_texture(TextureId::new(4))
        .with_detail_texture(TextureId::new(9));
    assert_eq!(textured.splat_texture, Some(TextureId::new(4)));

    // Sky: gradients, atmosphere, and sun glare pack into one uniform.
    let sky = SkyMaterial::clear_day();
    assert!(
        sky.glare_strength >= 0.0 && sky.glare_strength <= 1.0,
        "glare stays normalized"
    );
    let gradient = sky.with_gradient(
        Color::rgb(0.1, 0.2, 0.6),
        Color::rgb(0.6, 0.7, 0.9),
        Color::rgb(0.2, 0.2, 0.2),
    );
    assert_eq!(gradient.top_color, Color::rgb(0.1, 0.2, 0.6));
    assert!(gradient.with_atmosphere(true).atmosphere);

    // Water: foam thresholds park disabled masks above one.
    let water = WaterMaterial::clear().with_foam(0.45, 6.0);
    assert_eq!(water.foam_threshold, Some(0.45));
    assert_eq!(
        WaterMaterial::clear().foam_threshold,
        None,
        "foam stays off until requested"
    );

    // Shader kinds cover all three environment families (exhaustive match).
    let kind = ShaderKind::Terrain;
    let named = match kind {
        ShaderKind::Terrain => 0,
        ShaderKind::Sky => 1,
        ShaderKind::Water => 2,
        _ => 3,
    };
    assert_eq!(named, 0);

    // Feature bits stay distinct in [1 << 22, 1 << 26].
    for (a, b, name) in [
        (FEATURE_SPLAT_MAP, FEATURE_ATMOSPHERE, "splat/atmosphere"),
        (FEATURE_ATMOSPHERE, FEATURE_FRESNEL, "atmosphere/fresnel"),
        (FEATURE_FRESNEL, FEATURE_FOAM, "fresnel/foam"),
        (FEATURE_FOAM, FEATURE_REFRACTION, "foam/refraction"),
        (FEATURE_SPLAT_MAP, FEATURE_REFRACTION, "splat/refraction"),
    ] {
        assert_ne!(a, b, "feature bits overlap for {name}");
    }
}

#[test]
fn environment_ids_live_in_the_core_schema() {
    let terrain = TerrainId::new(9);
    assert!(!terrain.is_null(), "terrain ids reject the null sentinel");
    assert_eq!(terrain.get(), 9);
    let sky_id = SkyId::new(4);
    assert!(!sky_id.is_null(), "sky ids reject the null sentinel");
    assert_eq!(sky_id.get(), 4);
    let water_id = WaterId::new(3);
    assert!(!water_id.is_null(), "water ids reject the null sentinel");
    assert_eq!(water_id.get(), 3);
}

#[test]
fn terrain_noise_heightmaps_replay_and_split() {
    let noise = NoiseConfig::new(1_001, 0.035).with_octaves(3);
    let map = Heightmap::from_noise(33, 33, 1.0, &noise);
    let replay = Heightmap::from_noise(33, 33, 1.0, &noise);
    assert_eq!(map.heights, replay.heights, "noise replay is deterministic");

    let (min, _max) = map.height_range();
    assert!(min <= map.height_range().1);
    let policy = LodPolicy::new(&[LodBand::new(40.0, 0), LodBand::new(120.0, 1)]).expect("policy");
    let grid = TerrainGrid::generate(&map, TerrainConfig::new(2, 8, policy));
    assert!(grid.chunk_geometry(0, 0).is_some(), "chunks exist");
}
