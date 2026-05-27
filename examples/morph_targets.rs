use scenix::{MorphTarget, Vec3, sphere_geometry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let geometry = sphere_geometry(1.0, 16, 8);
    let deltas = geometry
        .positions
        .iter()
        .map(|position| Vec3::new(0.0, position.y.max(0.0) * 0.12, 0.0))
        .collect();
    let smile = MorphTarget::new("lift upper hemisphere")
        .positions_delta(deltas)
        .weight(0.5);

    smile.validate(geometry.vertex_count())?;
    println!(
        "morph target '{}' is valid for {} vertices",
        smile.name,
        geometry.vertex_count()
    );

    Ok(())
}
