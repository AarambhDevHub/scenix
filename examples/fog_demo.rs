use scenix::{Color, Fog, SceneGraph};

fn main() {
    let mut scene = SceneGraph::new();
    scene.set_fog(Some(Fog::linear(8.0, 35.0, Color::from_hex(0xAFC7E8))));
    println!("linear fog: {:?}", scene.fog());

    scene.set_fog(Some(Fog::exponential(0.035, Color::from_hex(0xDCEBFF))));
    println!("exponential fog: {:?}", scene.fog());
}
