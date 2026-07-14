use scenix_input::{GamepadAxis, InputState, KeyCode, PointerButton};
use scenix_math::{Quat, Transform, Vec2, Vec3};

use crate::{FlyController, OrthographicCamera, PerspectiveCamera, clamp};

fn input_look_delta(input: &InputState, dt: f32) -> Vec2 {
    let pointer = if input.pointer_lock.locked {
        input.pointer_lock.delta
    } else if input.pointer.is_pressed(PointerButton::Left) {
        input.pointer.delta
    } else {
        Vec2::ZERO
    };
    let touch = if input.gesture.contact_count == 1 {
        input.gesture.pan_delta
    } else {
        Vec2::ZERO
    };
    let gamepad = input
        .gamepads
        .connected()
        .next()
        .map_or(Vec2::ZERO, |(_, pad)| {
            Vec2::new(pad.axis(GamepadAxis::RightX), pad.axis(GamepadAxis::RightY))
        });
    pointer + touch + gamepad * (180.0 * dt.max(0.0))
}

fn zoom_factor(input: &InputState, sensitivity: f32) -> f32 {
    let pinch = if input.gesture.contact_count >= 2 {
        -input.gesture.pinch_delta
    } else {
        0.0
    };
    (1.0 + (input.scroll_delta + pinch) * sensitivity).max(0.01)
}

/// Quaternion-based orbit control suitable for model viewers.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArcballController {
    /// Point around which the camera rotates.
    pub target: Vec3,
    /// Camera distance from the target.
    pub distance: f32,
    /// Current camera orientation.
    pub rotation: Quat,
    /// Minimum distance.
    pub min_distance: f32,
    /// Maximum distance.
    pub max_distance: f32,
    /// Rotation radians per logical pixel.
    pub rotate_sensitivity: f32,
    /// Zoom response per scroll/pinch unit.
    pub zoom_sensitivity: f32,
}

impl ArcballController {
    /// Creates an arcball looking toward `target` from positive Z.
    pub fn new(target: Vec3, distance: f32) -> Self {
        Self {
            target,
            distance: distance.max(0.001),
            rotation: Quat::IDENTITY,
            min_distance: 0.001,
            max_distance: 1.0e9,
            rotate_sensitivity: 0.005,
            zoom_sensitivity: 0.1,
        }
    }

    /// Updates rotation and zoom from an aggregate input snapshot.
    pub fn update_from_input(&mut self, input: &InputState, dt: f32) -> Transform {
        let delta = input_look_delta(input, dt);
        if delta.length_squared() > crate::EPSILON {
            let yaw = Quat::from_axis_angle(Vec3::Y, -delta.x * self.rotate_sensitivity);
            let right = self.rotation.mul_vec3(Vec3::X).normalize();
            let pitch = Quat::from_axis_angle(right, -delta.y * self.rotate_sensitivity);
            self.rotation = (yaw * pitch * self.rotation).normalize();
        }
        self.distance *= zoom_factor(input, self.zoom_sensitivity);
        self.distance = clamp(
            self.distance,
            self.min_distance.max(0.001),
            self.max_distance.max(self.min_distance.max(0.001)),
        );
        self.camera_transform()
    }

    /// Returns the camera world transform.
    pub fn camera_transform(&self) -> Transform {
        let position = self.target + self.rotation.mul_vec3(Vec3::Z) * self.distance;
        Transform::looking_at(position, self.target, self.rotation.mul_vec3(Vec3::Y))
    }

    /// Applies the current pose to a perspective camera.
    pub fn apply_to_perspective(&self, camera: &mut PerspectiveCamera) {
        let transform = self.camera_transform();
        camera.position = transform.translation;
        camera.target = self.target;
        camera.up = transform.up();
    }
}

impl Default for ArcballController {
    fn default() -> Self {
        Self::new(Vec3::ZERO, 5.0)
    }
}

/// Unconstrained orbit, roll, pan, and zoom control.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrackballController {
    /// Underlying quaternion orbit state.
    pub arcball: ArcballController,
    /// Pan units per logical pixel at distance one.
    pub pan_sensitivity: f32,
    /// Roll sensitivity for two-finger rotation.
    pub roll_sensitivity: f32,
}

impl TrackballController {
    /// Creates a trackball around a target.
    pub fn new(target: Vec3, distance: f32) -> Self {
        Self {
            arcball: ArcballController::new(target, distance),
            pan_sensitivity: 0.001,
            roll_sensitivity: 1.0,
        }
    }

    /// Updates the trackball from pointer, touch, and gamepad input.
    pub fn update_from_input(&mut self, input: &InputState, dt: f32) -> Transform {
        if input.gesture.contact_count >= 2 {
            let transform = self.arcball.camera_transform();
            let scale = self.arcball.distance * self.pan_sensitivity;
            self.arcball.target += transform.right() * (-input.gesture.pan_delta.x * scale);
            self.arcball.target += transform.up() * (input.gesture.pan_delta.y * scale);
            if input.gesture.rotation_delta != 0.0 {
                let roll = Quat::from_axis_angle(
                    transform.forward(),
                    input.gesture.rotation_delta * self.roll_sensitivity,
                );
                self.arcball.rotation = (roll * self.arcball.rotation).normalize();
            }
        }
        self.arcball.update_from_input(input, dt)
    }

    /// Returns the current camera transform.
    pub fn camera_transform(&self) -> Transform {
        self.arcball.camera_transform()
    }

    /// Applies the current pose to a perspective camera.
    pub fn apply_to_perspective(&self, camera: &mut PerspectiveCamera) {
        self.arcball.apply_to_perspective(camera);
    }
}

impl Default for TrackballController {
    fn default() -> Self {
        Self::new(Vec3::ZERO, 5.0)
    }
}

/// World-up map/navigation control with orbit, pan, and zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapController {
    /// Ground-plane focus point.
    pub target: Vec3,
    /// Distance from the target.
    pub distance: f32,
    /// Heading around world Y.
    pub heading: f32,
    /// Downward tilt in radians.
    pub tilt: f32,
    /// Minimum tilt.
    pub min_tilt: f32,
    /// Maximum tilt.
    pub max_tilt: f32,
    /// Minimum distance.
    pub min_distance: f32,
    /// Maximum distance.
    pub max_distance: f32,
    /// Rotation sensitivity.
    pub rotate_sensitivity: f32,
    /// Pan sensitivity.
    pub pan_sensitivity: f32,
    /// Zoom sensitivity.
    pub zoom_sensitivity: f32,
}

impl MapController {
    /// Creates a map controller above a target.
    pub fn new(target: Vec3, distance: f32) -> Self {
        Self {
            target,
            distance: distance.max(0.001),
            heading: 0.0,
            tilt: 0.8,
            min_tilt: 0.05,
            max_tilt: core::f32::consts::FRAC_PI_2 - 0.01,
            min_distance: 0.01,
            max_distance: 1.0e9,
            rotate_sensitivity: 0.005,
            pan_sensitivity: 0.001,
            zoom_sensitivity: 0.1,
        }
    }

    /// Updates map navigation.
    pub fn update_from_input(&mut self, input: &InputState, _dt: f32) -> Transform {
        if input.pointer.is_pressed(PointerButton::Left) && input.gesture.contact_count == 0 {
            self.heading -= input.pointer.delta.x * self.rotate_sensitivity;
            self.tilt -= input.pointer.delta.y * self.rotate_sensitivity;
        }
        let pan = if input.gesture.contact_count >= 2
            || input.pointer.is_pressed(PointerButton::Right)
            || input.pointer.is_pressed(PointerButton::Middle)
        {
            if input.gesture.contact_count >= 2 {
                input.gesture.pan_delta
            } else {
                input.pointer.delta
            }
        } else {
            Vec2::ZERO
        };
        if pan.length_squared() > crate::EPSILON {
            let rotation = Quat::from_axis_angle(Vec3::Y, self.heading);
            let right = rotation.mul_vec3(Vec3::X);
            let forward = rotation.mul_vec3(Vec3::NEG_Z);
            let scale = self.distance * self.pan_sensitivity;
            self.target += right * (-pan.x * scale) + forward * (pan.y * scale);
        }
        self.distance *= zoom_factor(input, self.zoom_sensitivity);
        self.tilt = clamp(self.tilt, self.min_tilt, self.max_tilt);
        self.distance = clamp(
            self.distance,
            self.min_distance.max(0.001),
            self.max_distance,
        );
        self.camera_transform()
    }

    /// Returns the map camera transform.
    pub fn camera_transform(&self) -> Transform {
        let rotation = Quat::from_axis_angle(Vec3::Y, self.heading)
            * Quat::from_axis_angle(Vec3::X, -self.tilt);
        let position = self.target + rotation.mul_vec3(Vec3::Z) * self.distance;
        Transform::looking_at(position, self.target, Vec3::Y)
    }

    /// Applies the pose to a perspective camera.
    pub fn apply_to_perspective(&self, camera: &mut PerspectiveCamera) {
        camera.position = self.camera_transform().translation;
        camera.target = self.target;
        camera.up = Vec3::Y;
    }

    /// Applies the pose and zoom to an orthographic camera.
    pub fn apply_to_orthographic(&self, camera: &mut OrthographicCamera) {
        let half_height = self.distance.max(0.001);
        let aspect = ((camera.right - camera.left) / (camera.top - camera.bottom).max(0.001)).abs();
        camera.left = -half_height * aspect;
        camera.right = half_height * aspect;
        camera.bottom = -half_height;
        camera.top = half_height;
        camera.position = self.camera_transform().translation;
        camera.target = self.target;
        camera.up = Vec3::Y;
    }
}

impl Default for MapController {
    fn default() -> Self {
        Self::new(Vec3::ZERO, 10.0)
    }
}

/// Ground-plane first-person control with optional vertical movement.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FirstPersonController {
    /// Camera position.
    pub position: Vec3,
    /// Yaw around world Y.
    pub yaw: f32,
    /// Pitch around local X.
    pub pitch: f32,
    /// Movement units per second.
    pub speed: f32,
    /// Look radians per logical pixel.
    pub sensitivity: f32,
    /// Absolute pitch limit.
    pub pitch_limit: f32,
    /// Whether Q/E and triggers move vertically.
    pub allow_vertical: bool,
}

impl FirstPersonController {
    /// Creates a controller at a world position.
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
            speed: 5.0,
            sensitivity: 0.003,
            pitch_limit: core::f32::consts::FRAC_PI_2 - 0.001,
            allow_vertical: false,
        }
    }

    /// Updates look and movement from aggregate input.
    pub fn update_from_input(&mut self, input: &InputState, dt: f32) -> Transform {
        let delta = input_look_delta(input, dt);
        self.yaw -= delta.x * self.sensitivity;
        self.pitch = clamp(
            self.pitch - delta.y * self.sensitivity,
            -self.pitch_limit,
            self.pitch_limit,
        );
        let rotation = self.rotation();
        let full_forward = rotation.mul_vec3(Vec3::NEG_Z);
        let forward = Vec3::new(full_forward.x, 0.0, full_forward.z).normalize();
        let right = Vec3::new(forward.z, 0.0, -forward.x);
        let mut movement = Vec3::ZERO;
        if input.keyboard.is_pressed(KeyCode::KeyW) || input.keyboard.is_pressed(KeyCode::ArrowUp) {
            movement += forward;
        }
        if input.keyboard.is_pressed(KeyCode::KeyS) || input.keyboard.is_pressed(KeyCode::ArrowDown)
        {
            movement -= forward;
        }
        if input.keyboard.is_pressed(KeyCode::KeyD)
            || input.keyboard.is_pressed(KeyCode::ArrowRight)
        {
            movement += right;
        }
        if input.keyboard.is_pressed(KeyCode::KeyA) || input.keyboard.is_pressed(KeyCode::ArrowLeft)
        {
            movement -= right;
        }
        if self.allow_vertical {
            if input.keyboard.is_pressed(KeyCode::KeyE) || input.keyboard.is_pressed(KeyCode::Space)
            {
                movement += Vec3::Y;
            }
            if input.keyboard.is_pressed(KeyCode::KeyQ) {
                movement -= Vec3::Y;
            }
        }
        if let Some((_, pad)) = input.gamepads.connected().next() {
            movement +=
                right * pad.axis(GamepadAxis::LeftX) - forward * pad.axis(GamepadAxis::LeftY);
        }
        if movement.length_squared() > crate::EPSILON {
            self.position += movement.normalize() * self.speed * dt.max(0.0);
        }
        self.camera_transform()
    }

    /// Returns the current orientation.
    pub fn rotation(&self) -> Quat {
        Quat::from_axis_angle(Vec3::Y, self.yaw) * Quat::from_axis_angle(Vec3::X, self.pitch)
    }

    /// Returns the current camera transform.
    pub fn camera_transform(&self) -> Transform {
        Transform::new(self.position, self.rotation(), Vec3::ONE)
    }

    /// Applies the pose to a perspective camera.
    pub fn apply_to_perspective(&self, camera: &mut PerspectiveCamera) {
        let rotation = self.rotation();
        camera.position = self.position;
        camera.target = self.position + rotation.mul_vec3(Vec3::NEG_Z);
        camera.up = rotation.mul_vec3(Vec3::Y);
    }
}

impl Default for FirstPersonController {
    fn default() -> Self {
        Self::new(Vec3::ZERO)
    }
}

/// Fly controller adapter that responds only to relative look input while locked.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PointerLockController {
    /// Reused fly movement and camera state.
    pub fly: FlyController,
    /// When true, unlocked calls retain pose without consuming input.
    pub require_lock: bool,
}

impl PointerLockController {
    /// Creates a pointer-lock controller at a position.
    pub fn new(position: Vec3) -> Self {
        Self {
            fly: FlyController::new(position),
            require_lock: true,
        }
    }

    /// Updates only when the pointer-lock requirement is satisfied.
    pub fn update_from_input(&mut self, input: &InputState, dt: f32) -> Transform {
        if self.require_lock && !input.pointer_lock.locked {
            return self.camera_transform();
        }
        self.fly.update_from_input(input, dt)
    }

    /// Returns the current camera transform.
    pub fn camera_transform(&self) -> Transform {
        Transform::new(self.fly.position, self.fly.rotation(), Vec3::ONE)
    }

    /// Applies the pose to a perspective camera.
    pub fn apply_to_perspective(&self, camera: &mut PerspectiveCamera) {
        self.fly.apply_to_perspective(camera);
    }
}

impl Default for PointerLockController {
    fn default() -> Self {
        Self::new(Vec3::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenix_input::{GamepadId, ViewportMetrics};

    #[test]
    fn arcball_zoom_is_clamped_and_finite() {
        let mut input = InputState::new(ViewportMetrics::default());
        input.on_scroll(-1000.0);
        let mut control = ArcballController::default();
        control.update_from_input(&input, 1.0 / 60.0);
        assert!(control.distance >= control.min_distance);
        assert!(control.camera_transform().translation.x.is_finite());
    }

    #[test]
    fn first_person_uses_keyboard_and_gamepad() {
        let mut input = InputState::default();
        input.on_key_down(KeyCode::KeyW);
        input.set_gamepad_connected(GamepadId(0), true);
        input.set_gamepad_axis(GamepadId(0), GamepadAxis::LeftX, 1.0);
        let mut control = FirstPersonController::default();
        control.update_from_input(&input, 1.0);
        assert!(control.position.length_squared() > 1.0);
    }

    #[test]
    fn pointer_lock_requirement_gates_motion() {
        let mut input = InputState::default();
        input.on_key_down(KeyCode::KeyW);
        let mut control = PointerLockController::default();
        control.update_from_input(&input, 1.0);
        assert_eq!(control.fly.position, Vec3::ZERO);
        input.set_pointer_locked(true);
        control.update_from_input(&input, 1.0);
        assert_ne!(control.fly.position, Vec3::ZERO);
    }
}
