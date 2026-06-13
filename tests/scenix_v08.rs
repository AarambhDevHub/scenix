#[cfg(all(feature = "helpers", feature = "raycaster"))]
use scenix::{AxesHelper, GridHelper, LineGeometry, Raycaster};

#[cfg(all(feature = "helpers", feature = "raycaster"))]
#[test]
fn facade_exports_v08_raycasting_and_helpers() {
    let raycaster = Raycaster::new();
    assert_eq!(raycaster.layers(), u32::MAX);

    let mut lines = LineGeometry::new();
    lines.merge(&GridHelper::new(2.0, 2).to_geometry());
    lines.merge(&AxesHelper::new(1.0).to_geometry());
    lines.validate().unwrap();
    assert_eq!(lines.segment_count(), 9);
}
