use scenix::{
    GroundedSkybox, SkyModel, SunPosition, Vec3, bake_ambient, bake_equirectangular,
    bake_light_probe,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A daytime profile with the sun high in the southeast sky.
    let day = SkyModel::clear_day();
    let sun = SunPosition::degrees(42.0, 115.0);
    let sun_dir = sun.direction();

    // Sky profiles are pure functions of direction: same tap, same color.
    let zenith = day.rgb_at(Vec3::Y, sun_dir);
    let repeat = day.rgb_at(Vec3::Y, sun_dir);
    assert_eq!(zenith, repeat);
    println!(
        "zenith sky: {:.3} {:.3} {:.3}",
        zenith[0], zenith[1], zenith[2]
    );

    let horizon = day.rgb_at(Vec3::new(1.0, 0.05, 0.0), sun_dir);
    println!(
        "horizon sky: {:.3} {:.3} {:.3}",
        horizon[0], horizon[1], horizon[2]
    );

    // Overcast haze is a tuning surface over the same model.
    let overcast = SkyModel::overcast_haze()
        .with_turbidity(7.0)
        .with_rayleigh(1.2)
        .with_mie(0.4, 0.9);
    let overcast_zenith = overcast.rgb_at(Vec3::Y, sun_dir);
    println!(
        "overcast zenith: {:.3} {:.3} {:.3} (turbidity {:.1})",
        overcast_zenith[0], overcast_zenith[1], overcast_zenith[2], overcast.turbidity
    );

    // Bake a 64×32 equirectangular environment map (deterministic).
    let baked = bake_equirectangular(&day, &sun, 64, 32);
    println!("baked {} equirect rows", baked.len());

    // Ambient and light-probe bakes roll the map into lighting taps.
    let ambient = bake_ambient(&baked, 64, 32);
    println!(
        "ambient sky: {:.3} {:.3} {:.3} / ground: {:.3} {:.3} {:.3}",
        ambient.sky[0],
        ambient.sky[1],
        ambient.sky[2],
        ambient.ground[0],
        ambient.ground[1],
        ambient.ground[2]
    );
    let probe = bake_light_probe(&day, &sun, 16, 16, 1.25).expect("probe projection succeeds");
    println!(
        "light probe carries {} SH taps of radiance",
        probe.sh_coefficients.len()
    );

    // Sun elevation drives the light's angle and warmth.
    let low = SunPosition::degrees(14.0, 200.0).sun_light(4.0);
    println!(
        "sunset light: warm={} direction=({:.2}, {:.2}, {:.2})",
        low.color.r > low.color.b,
        low.direction.x,
        low.direction.y,
        low.direction.z
    );

    // Grounded skyboxes fade a dome into a flat ground patch.
    let grounded = GroundedSkybox::new(
        scenix::Color::rgb(0.08, 0.24, 0.72),
        scenix::Color::rgb(0.86, 0.64, 0.42),
        scenix::Color::rgb(0.12, 0.12, 0.11),
    );
    let ground = grounded.rgb_at(Vec3::new(0.0, -1.0, 0.0));
    println!(
        "grounded band: {:.3} {:.3} {:.3}",
        ground[0], ground[1], ground[2]
    );

    Ok(())
}
