//! Water CPU integration tests (scenix v1.8.0): waves, surfaces, fresnel,
//! foam, underwater tinting, and reflection helpers.

use scenix_camera::PerspectiveCamera;
use scenix_math::Vec3;
use scenix_water::{
    FoamParams, FresnelParams, GerstnerWave, ReflectionSetup, UnderwaterView, WaterConfig,
    WaterSurface, WaveField, mirror_transform, reflected_camera,
};

#[test]
fn gerstner_waves_displace_forwards_and_leave_flat_fields_flat() {
    let flat = WaveField::new();
    let idle = flat.evaluate(gerstner_point(0.0, 0.0), 0.0);
    assert_eq!(idle.position.y, 0.0);
    assert!((idle.normal.y - 1.0).abs() < 1.0e-6);
    assert_eq!(idle.crest, 0.0);

    let active = WaveField::ocean_shore();
    let risen = active.evaluate(gerstner_point(3.0, -1.0), 0.9);
    // Gerstner XZ displacement compresses horizontal coordinates: the
    // sampled x must differ from the resting x when waves are active.
    assert!(
        (risen.position.x - 3.0).abs() > 1.0e-6 || (risen.position.y).abs() > 1.0e-6,
        "waves displace the sampled point"
    );
    assert!(risen.crest >= 0.0 && risen.crest <= 1.0);

    // Wavelength → wave-number contract.
    let wave = GerstnerWave::new(scenix_math::Vec2::new(1.0, 0.0), 0.5, 3.0);
    assert!((wave.wave_number() - core::f32::consts::TAU / 3.0).abs() < 1.0e-5);
    assert!(wave.phase_speed() > 1.0);
}

#[test]
fn surface_geometry_matches_wave_heights() {
    let surface = WaterSurface::new(WaveField::ocean_shore())
        .with_config(WaterConfig::new(24.0, 24.0, 6, 1.5));
    let geometry = surface.geometry(0.6);

    assert_eq!(geometry.vertex_count(), 49);
    assert_eq!(geometry.triangle_count(), 72);
    assert_eq!(geometry.normals.len(), geometry.positions.len());
    assert_eq!(geometry.uvs.len(), geometry.positions.len());
    assert!(
        geometry.colors.is_empty(),
        "water uses uniforms, not splats"
    );

    for vertex in geometry.positions.iter().take(6) {
        let expected = surface.height_at(vertex.x, vertex.z, 0.6);
        assert!(
            (vertex.y - expected).abs() < 1.0e-4,
            "vertex height {vertex:?} drifts from {expected}"
        );
    }
    for normal in geometry.normals.iter().take(6) {
        assert!((normal.length() - 1.0).abs() < 5.0e-3);
    }
}

#[test]
fn foam_and_fresnel_track_physical_shapes() {
    let foam = FoamParams::shoreline().with_threshold(0.5);
    assert_eq!(foam.mask(0.0), 0.0);
    assert_eq!(
        foam.mask(0.5),
        0.0,
        "the gate sits exactly at the threshold"
    );
    assert!(foam.mask(0.55) > 0.0, "coverage begins past the gate");
    assert!(
        foam.mask(0.75) > foam.mask(0.55),
        "smoothstep coverage grows toward crests"
    );
    assert_eq!(foam.mask(1.0), 1.0, "full crests saturate at coverage 1");
    assert_eq!(foam.masks_over(&[0.2, 0.9]).len(), 2);

    let water = FresnelParams::water();
    assert!((water.reflectance(1.0) - water.base_reflectance).abs() < 1.0e-6);
    assert!(water.reflectance(0.0) > 0.95, "grazing angles mirror fully");
    let dimmed = water.with_sheen(0.5).reflectance(1.0);
    assert!((dimmed - water.base_reflectance * 0.5).abs() < 1.0e-6);
}

#[test]
fn underwater_tint_absorbs_red_first_and_fades_to_murk() {
    let view = UnderwaterView::clear_pool();
    let shallow = view.transmittance(1.0);
    let deep = view.transmittance(6.0);
    assert!(shallow[0] < shallow[2], "red absorbs before blue");
    for (near_channel, far_channel) in shallow.iter().zip(&deep) {
        assert!(far_channel < near_channel);
    }

    let white = scenix_core::Color::WHITE;
    let close = view.view_tint(white, 0.4);
    let distant = view.view_tint(white, 9.0);
    let murk = view.murk_color.to_array();
    // Distance pulls every channel toward the murk tint.
    let drift_near: f32 = close.iter().zip(murk).map(|(v, m)| (v - m).abs()).sum();
    let drift_far: f32 = distant.iter().zip(murk).map(|(v, m)| (v - m).abs()).sum();
    assert!(drift_far < drift_near, "distance drives the fade into murk");
    assert!(distant[0] < close[0], "distance mutes the red channel");
    // The additive murk tint always contributes non-negative energy.
    let murkier = view.with_murk(1.0).view_tint(white, 3.0);
    let clear = view.with_murk(0.05).view_tint(white, 3.0);
    assert!(
        murkier.iter().zip(clear).all(|(&a, b)| a >= b - 1.0e-6),
        "denser murk never darkens a channel: {murkier:?} vs {clear:?}"
    );
}

#[test]
fn reflection_helpers_mirror_cameras_and_state() {
    // The mirror transform flips points through plane y = 2.
    let matrix = mirror_transform(2.0).to_mat4();
    use scenix_math::Vec4;
    let flipped = matrix.mul_vec4(Vec4::new(1.0, 5.0, -1.0, 1.0));
    assert!(
        (flipped.y - (2.0 * 2.0 - 5.0)).abs() < 1.0e-5,
        "5 → −1 across plane y=2"
    );
    assert!((flipped.x - 1.0).abs() < 1.0e-5 && (flipped.z + 1.0).abs() < 1.0e-5);
    // Below-plane points bounce upward.
    let trough = matrix.mul_vec4(Vec4::new(0.0, 0.5, 0.0, 1.0));
    assert!((trough.y - 3.5).abs() < 1.0e-5);

    let camera = PerspectiveCamera::new(50.0, 1.6, 0.2, 300.0)
        .position(Vec3::new(2.0, 10.0, 6.0))
        .target(Vec3::new(0.0, 3.0, 0.0))
        .up(Vec3::Y);
    let mirrored = reflected_camera(&camera, 2.0);
    assert!((mirrored.position.y - (-6.0)).abs() < 1.0e-6, "10 → 4 − 10");
    assert!((mirrored.target.y - (1.0)).abs() < 1.0e-6, "3 → 4 − 3");
    assert!((mirrored.up.y + 1.0).abs() < 1.0e-6);
    assert!((mirrored.position.z - 6.0).abs() < 1.0e-6);

    // Reflection setup keeps target plumbing intact.
    let setup = ReflectionSetup::minimal(21)
        .with_refraction(22)
        .with_size(256);
    assert_eq!(setup.reflection_texture, 21);
    assert_eq!(setup.refraction_texture, 22);
    assert_eq!(setup.size, 256);
}

#[test]
fn surfaces_render_with_clamped_strength_knobs_and_defaults() {
    let surface = WaterSurface::new(WaveField::calm()).with_strengths(3.0, -2.0);
    assert_eq!(surface.reflection_strength, 1.0);
    assert_eq!(surface.refraction_strength, 0.0);
    assert_eq!(
        surface.config,
        WaterConfig::new(64.0, 64.0, 32, 0.0),
        "default patch drives the config"
    );

    // Foam masks on the resting plane behave like the raw params.
    let mask = surface.foam_at(1.0, 1.0, 2.5);
    let crest = surface
        .waves
        .evaluate(scenix_math::Vec2::new(1.0, 1.0), 2.5)
        .crest;
    assert!((mask - surface.foam.mask(crest)).abs() < 1.0e-6);
}

#[inline]
fn gerstner_point(x: f32, z: f32) -> scenix_math::Vec2 {
    scenix_math::Vec2::new(x, z)
}
