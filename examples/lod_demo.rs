use scenix::{LodGroup, MeshId};

fn main() {
    let lod = LodGroup::new(vec![
        (4.0, MeshId::new(1)),
        (12.0, MeshId::new(2)),
        (40.0, MeshId::new(3)),
    ]);

    for distance in [2.0, 8.0, 20.0, 80.0] {
        println!("distance {distance:>4}: mesh {:?}", lod.select(distance));
    }
}
