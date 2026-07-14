use scenix::{GizmoGeometry, Ray3, TransformGizmoHelper, TransformMode, TransformSpace, Vec3};

fn main() {
    let mut helper = TransformGizmoHelper::new(Vec3::ZERO, TransformMode::Translate);
    helper.space = TransformSpace::World;
    helper.size = 2.0;

    let mut geometry = GizmoGeometry::default();
    helper.write_geometry(&mut geometry);
    let ray = Ray3::new(Vec3::new(1.0, 0.04, 4.0), Vec3::NEG_Z);
    println!(
        "{} segments, picked {:?}",
        geometry.lines.segment_count(),
        geometry.hit_test(ray)
    );
}
