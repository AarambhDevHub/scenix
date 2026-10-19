use scenix::{
    FoamParams, FresnelParams, GerstnerWave, PerspectiveCamera, UnderwaterView, Vec2, Vec3,
    WaterConfig, WaterSurface, WaveField, mirror_transform, reflected_camera,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The ocean-shore preset carries three stacked Gerstner waves.
    let field = WaveField::ocean_shore();
    let surface = WaterSurface::new(field).with_config(WaterConfig::new(24.0, 24.0, 8, 1.2));

    // The surface rebuilds as displaced geometry at any timestamp.
    let geometry = surface.geometry(0.85);
    println!(
        "surface: {} verts / {} triangles at t = 0.85 s",
        geometry.vertex_count(),
        geometry.triangle_count()
    );

    // Height sampling matches the geometry displacement exactly.
    for column in [0.0_f32, 6.0, 12.0, 18.0] {
        let height = surface.height_at(column, column, 0.85);
        let crest = surface.waves.evaluate(Vec2::new(column, column), 0.85);
        println!(
            "height at ({column:.0}, {column:.0}): {height:.3} (crest {:+.3})",
            crest.crest
        );
    }

    // Wave math: wavelength → wave number → phase speed.
    let wave = GerstnerWave::new(Vec2::new(1.0, 0.0), 0.5, 6.0);
    println!(
        "6 m wave: k = {:.3} rad/m, phase speed {:.2} m/s",
        wave.wave_number(),
        wave.phase_speed()
    );

    // Fresnel ramps reflectance with incidence angle.
    let fresnel = FresnelParams::water();
    for theta in [0.0_f32, 0.35, 0.7, 1.0] {
        println!(
            "reflectance(n·l = {:>4.2}): {:.3}",
            theta,
            fresnel.reflectance(theta)
        );
    }

    // Foam gates crest energy behind a threshold.
    let foam = FoamParams::shoreline().with_threshold(0.5);
    println!(
        "foam: crest 0.4 → {:.2}, crest 0.9 → {:.2}",
        foam.mask(0.4),
        foam.mask(0.9)
    );

    // Sub-surface camera tinting absorbs red first.
    let view = UnderwaterView::clear_pool();
    let shallow = view.transmittance(1.0);
    let deep = view.transmittance(8.0);
    println!(
        "transmittance 1 m: ({:.2}, {:.2}, {:.2}) → 8 m: ({:.2}, {:.2}, {:.2})",
        shallow[0], shallow[1], shallow[2], deep[0], deep[1], deep[2]
    );

    // Reflection helpers mirror a camera through the water plane.
    let camera = PerspectiveCamera::new(50.0, 1.6, 0.2, 300.0)
        .position(Vec3::new(2.0, 10.0, 6.0))
        .target(Vec3::new(0.0, 3.0, 0.0))
        .up(Vec3::Y);
    let mirrored = reflected_camera(&camera, 2.0);
    println!(
        "mirrored camera: y {:.3} → {:.3}",
        camera.position.y, mirrored.position.y
    );
    let matrix = mirror_transform(2.0).to_mat4();
    let flipped = matrix.mul_vec4(scenix::Vec4::new(1.0, 5.0, -1.0, 1.0));
    println!("plane flip: y = {:.3}", flipped.y);

    Ok(())
}
