#![cfg(feature = "serde")]

use scenix_core::{Color, NodeId};

#[test]
fn core_types_round_trip_with_serde_shape() {
    let id = NodeId::new(9);
    let color = Color::rgba(0.1, 0.2, 0.3, 0.4);

    let id_json = serde_json::to_string(&id).unwrap();
    let color_json = serde_json::to_string(&color).unwrap();

    assert_eq!(serde_json::from_str::<NodeId>(&id_json).unwrap(), id);
    assert_eq!(serde_json::from_str::<Color>(&color_json).unwrap(), color);
}
