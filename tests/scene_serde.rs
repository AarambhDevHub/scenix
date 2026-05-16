#![cfg(feature = "serde")]

use scenix_core::{MeshId, TextureId};
use scenix_scene::{BillboardMode, LodGroup, NodeKind, SceneNode, Sprite};

#[test]
fn scene_data_types_round_trip_with_serde() {
    let sprite = Sprite::new(1.0, 2.0, TextureId::new(5)).billboard(BillboardMode::AxisAlignedY);
    let node = SceneNode::sprite("sprite", sprite).layer(0b1010);
    let lod = LodGroup::new(vec![(10.0, MeshId::new(1)), (50.0, MeshId::new(2))]);

    let node_json = serde_json::to_string(&node).unwrap();
    let lod_json = serde_json::to_string(&lod).unwrap();

    let decoded_node: SceneNode = serde_json::from_str(&node_json).unwrap();
    let decoded_lod: LodGroup = serde_json::from_str(&lod_json).unwrap();

    assert_eq!(decoded_node, node);
    assert_eq!(decoded_lod, lod);
    assert_eq!(decoded_node.kind, NodeKind::Sprite(sprite));
}
