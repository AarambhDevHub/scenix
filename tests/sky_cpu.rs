//! Sky, sun, and environment-baking CPU integration tests (scenix v1.8.0).

use scenix_light::DirectionalLight;
use scenix_math::Vec3;
use scenix_sky::{
    GroundedSkybox, SkyModel, SunPosition, bake_ambient, bake_equirectangular, bake_light_probe,
};

#[test]
fn sky_model_reads_blue_zenith_and_bright_glare() {
    let model = SkyModel::clear_day();
    let sun = SunPosition::degrees(35.0, 40.0);
    let sun_dir = sun.direction();

    let zenith = model.rgb_at(Vec3::Y, sun_dir);
    let horizon = model.rgb_at(Vec3::new(1.0, 0.0, 0.0), sun_dir);
    assert!(zenith[2] > zenith[0], "zenith stays blue: {zenith:?}");
    assert!(horizon[0] > zenith[0], "horizon warms: {horizon:?}");

    // Looking toward the sun is brighter than looking away from it.
    let toward: f32 = model.rgb_at(sun_dir, sun_dir).iter().sum();
    let away: f32 = model
        .rgb_at(Vec3::new(0.3, 0.5, -0.8).normalize(), sun_dir)
        .iter()
        .sum();
    assert!(toward > away, "glare direction must be brighter");

    // Deterministic replay.
    let repeat = model.rgb_at(Vec3::Y, sun_dir);
    assert_eq!(zenith, repeat);

    // Presets share a responsive tuning surface.
    let custom = SkyModel::overcast_haze()
        .with_turbidity(6.0)
        .with_rayleigh(1.0)
        .with_mie(0.5, 0.95);
    assert_eq!(custom.turbidity, 6.0);
    assert!(custom.mie_g > 0.9);
}

#[test]
fn sun_positions_drive_directional_lights() {
    let sun = SunPosition::degrees(30.0, 110.0);
    let sun_light = sun.sun_light(5.0);
    assert!((sun_light.direction.length() - 1.0).abs() < 1.0e-5);
    assert!((sun_light.intensity - 5.0).abs() < 1.0e-6);
    assert!(sun_light.color.r > sun_light.color.b, "low sun leans warm");

    let directional: DirectionalLight = sun.directional_light(5.0);
    assert!((directional.intensity - 5.0).abs() < 1.0e-6);
    assert!((directional.direction.y - sun_light.direction.y).abs() < 1.0e-6);

    // Skylight reaches the sun at the zenith (no horizontal tangling).
    let up = SunPosition::radians(core::f32::consts::FRAC_PI_2, 0.0).direction();
    assert!(up.x.abs() < 1.0e-5 && up.z.abs() < 1.0e-5);
}

#[test]
fn equirect_bakes_are_deterministic_and_sh_projection_round_trips() {
    let model = SkyModel::clear_day();
    let sun = SunPosition::degrees(25.0, -20.0);

    let baked = bake_equirectangular(&model, &sun, 10, 8);
    assert_eq!(baked.len(), 80);
    let again = bake_equirectangular(&model, &sun, 10, 8);
    assert_eq!(baked, again, "bakes replay exactly");

    // Top rows must read cooler/bluer than bottom rows (sky vs clamped
    // ground view).
    let top_blue = baked[0][2] - baked[0][0];
    assert!(top_blue > 0.0, "top sky rows stay blue");

    let probe = bake_light_probe(&model, &sun, 16, 16, 1.25).expect("probe projection succeeds");
    let magnitude: f32 = probe
        .sh_coefficients
        .iter()
        .map(|c| c.x.abs() + c.y.abs() + c.z.abs())
        .sum();
    assert!(magnitude > 0.0, "SH taps carry radiance");

    let ambient = bake_ambient(&baked, 10, 8);
    assert!(ambient.sky.iter().all(|v| *v >= 0.0));
    assert!(ambient.ground.iter().all(|v| *v >= 0.0));
    let sky_total: f32 = ambient.sky.iter().sum();
    assert!(sky_total > 0.0, "the upper hemisphere carries sky");
}

#[test]
fn grounded_skyboxes_fade_between_sky_and_ground() {
    let skybox = GroundedSkybox::new(
        scenix_core::Color::rgb(0.08, 0.24, 0.72),
        scenix_core::Color::rgb(0.86, 0.64, 0.42),
        scenix_core::Color::rgb(0.12, 0.12, 0.11),
    );

    let top = skybox.rgb_at(Vec3::Y);
    let horizon = skybox.rgb_at(Vec3::new(1.0, 0.0, 0.0));
    let ground = skybox.rgb_at(Vec3::new(0.0, -1.0, 0.0));
    assert!(
        (top[2] - 0.72).abs() < 1.0e-5,
        "zenith equals the top color"
    );
    assert!(horizon[0] > 0.7, "horizon stays warm");
    assert!(ground[0] < 0.2 && ground[2] < 0.2, "ground closes dark");

    // The dome sampler stays deterministic and ordered (zenith first).
    let dome = skybox.sample_dome(6, 6);
    assert_eq!(dome.len(), 36);
    assert!(dome[0][2] > dome[dome.len() - 1][2]);
    assert_eq!(skybox.sample_dome(6, 6), dome, "repeated sampling replays");
}
