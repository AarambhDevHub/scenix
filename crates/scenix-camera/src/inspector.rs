use scenix_core::{Inspectable, InspectorId, InspectorItem, InspectorSnapshot};

use crate::{
    ArcballController, FirstPersonController, MapController, OrthographicCamera, PerspectiveCamera,
    TrackballController,
};

impl Inspectable for PerspectiveCamera {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(InspectorId(1), "Perspective Camera", "camera")
                .field("fov_radians", self.fov_y)
                .field("aspect", self.aspect)
                .field("near", self.near)
                .field("far", self.far)
                .field("position", self.position)
                .field("target", self.target),
        );
    }
}

impl Inspectable for OrthographicCamera {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(InspectorId(1), "Orthographic Camera", "camera")
                .field("left", self.left)
                .field("right", self.right)
                .field("bottom", self.bottom)
                .field("top", self.top)
                .field("near", self.near)
                .field("far", self.far)
                .field("position", self.position)
                .field("target", self.target),
        );
    }
}

macro_rules! inspect_orbit_control {
    ($ty:ty, $label:literal, $target:expr, $distance:expr) => {
        impl Inspectable for $ty {
            fn inspect(&self, snapshot: &mut InspectorSnapshot) {
                snapshot.push(
                    InspectorItem::new(InspectorId(1), $label, "camera_control")
                        .field("target", ($target)(self))
                        .field("distance", ($distance)(self)),
                );
            }
        }
    };
}

inspect_orbit_control!(
    ArcballController,
    "Arcball",
    |value: &ArcballController| value.target,
    |value: &ArcballController| value.distance
);
inspect_orbit_control!(
    TrackballController,
    "Trackball",
    |value: &TrackballController| value.arcball.target,
    |value: &TrackballController| value.arcball.distance
);
inspect_orbit_control!(
    MapController,
    "Map",
    |value: &MapController| value.target,
    |value: &MapController| value.distance
);

impl Inspectable for FirstPersonController {
    fn inspect(&self, snapshot: &mut InspectorSnapshot) {
        snapshot.push(
            InspectorItem::new(InspectorId(1), "First Person", "camera_control")
                .field("position", self.position)
                .field("yaw", self.yaw)
                .field("pitch", self.pitch)
                .field("speed", self.speed),
        );
    }
}
