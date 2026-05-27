use scenix::{BillboardMode, SceneGraph, SceneNode, Sprite, TextureId, Transform, Vec3};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scene = SceneGraph::with_capacity(128);
    let texture_id = TextureId::new(1);

    for index in 0..64 {
        let angle = index as f32 * 0.37;
        let radius = 1.0 + (index % 8) as f32 * 0.2;
        let position = Vec3::new(
            angle.cos() * radius,
            index as f32 * 0.02,
            angle.sin() * radius,
        );
        scene.add(
            SceneNode::sprite(
                format!("particle-{index}"),
                Sprite::new(0.12, 0.12, texture_id).billboard(BillboardMode::FaceCamera),
            )
            .transform(Transform::from_translation(position)),
        );
    }

    scene.update_world_transforms();
    println!(
        "created {} billboard particles",
        scene.iter_depth_first().count()
    );
    Ok(())
}
