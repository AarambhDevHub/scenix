#[cfg(feature = "animato")]
#[test]
fn facade_exports_animato_bridge_types() {
    let mut track = scenix::ScalarTrack::tween(0.0, 1.0, 1.0);
    track.update(1.0);
    assert_eq!(track.value(), 1.0);
}

#[cfg(feature = "wasm")]
#[test]
fn facade_exports_wasm_mapping_helpers() {
    assert_eq!(
        scenix::key_code_from_dom("KeyW"),
        Some(scenix::KeyCode::KeyW)
    );
    assert_eq!(scenix::clamp_canvas_size(0, 2), (1, 2));
}
