use scenix_input::{KeyCode, KeyboardState, PointerButton, PointerState};
use scenix_math::{Quat, Spherical, Transform, Vec2, Vec3};

use crate::{PerspectiveCamera, clamp};

/// Orbit-style camera controller.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrbitController {
    /// Orbit target.
    pub target: Vec3,
    /// Distance from target.
    pub distance: f32,
    /// Azimuth angle in radians.
    pub theta: f32,
    /// Polar angle in radians.
    pub phi: f32,
    /// Minimum distance.
    pub min_distance: f32,
    /// Maximum distance.
    pub max_distance: f32,
    /// Minimum polar angle.
    pub min_polar_angle: f32,
    /// Maximum polar angle.
    pub max_polar_angle: f32,
    /// Rotation sensitivity in radians per pixel.
    pub rotate_sensitivity: f32,
    /// Zoom sensitivity per scroll unit.
    pub zoom_sensitivity: f32,
    /// Pan sensitivity in world units per pixel at distance 1.
    pub pan_sensitivity: f32,
    /// Damping coefficient reserved for renderer loops.
    pub damping: f32,
}

/// Fly-style camera controller.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlyController {
    /// Camera position.
    pub position: Vec3,
    /// Yaw angle in radians.
    pub yaw: f32,
    /// Pitch angle in radians.
    pub pitch: f32,
    /// Movement speed in units per second.
    pub speed: f32,
    /// Multiplier when either shift key is pressed.
    pub fast_multiplier: f32,
    /// Pointer-look sensitivity in radians per pixel.
    pub sensitivity: f32,
    /// Absolute pitch clamp in radians.
    pub pitch_limit: f32,
}

impl OrbitController {
    /// Creates an orbit controller looking at a target.
    #[inline]
    pub fn new(target: Vec3, distance: f32) -> Self {
        Self {
            target,
            distance: distance.max(0.001),
            theta: 0.0,
            phi: core::f32::consts::FRAC_PI_2,
            min_distance: 0.001,
            max_distance: 1.0e9,
            min_polar_angle: 0.001,
            max_polar_angle: core::f32::consts::PI - 0.001,
            rotate_sensitivity: 0.005,
            zoom_sensitivity: 0.1,
            pan_sensitivity: 0.001,
            damping: 0.0,
        }
    }

    /// Applies drag rotation.
    pub fn on_drag(&mut self, delta: Vec2, _dt: f32) {
        self.theta -= delta.x * self.rotate_sensitivity;
        self.phi -= delta.y * self.rotate_sensitivity;
        self.clamp_state();
    }

    /// Applies scroll zoom.
    pub fn on_scroll(&mut self, delta: f32, _dt: f32) {
        let scale = 1.0 + delta * self.zoom_sensitivity;
        self.distance *= scale.max(0.001);
        self.clamp_state();
    }

    /// Applies local camera-plane panning.
    pub fn on_pan(&mut self, delta: Vec2, _dt: f32) {
        let transform = self.camera_transform();
        let pan_scale = self.distance * self.pan_sensitivity;
        self.target += transform.right() * (-delta.x * pan_scale);
        self.target += transform.up() * (delta.y * pan_scale);
    }

    /// Consumes pointer state. Left drag orbits, right/middle drag pans.
    pub fn update_from_pointer(&mut self, pointer: PointerState, scroll_delta: f32, dt: f32) {
        if pointer.is_pressed(PointerButton::Left) {
            self.on_drag(pointer.delta, dt);
        } else if pointer.is_pressed(PointerButton::Right)
            || pointer.is_pressed(PointerButton::Middle)
        {
            self.on_pan(pointer.delta, dt);
        }
        if scroll_delta != 0.0 {
            self.on_scroll(scroll_delta, dt);
        }
    }

    /// Clamps distance and polar angle.
    #[inline]
    pub fn update(&mut self, _dt: f32) {
        self.clamp_state();
    }

    /// Returns the camera transform.
    pub fn camera_transform(&self) -> Transform {
        let offset = Spherical::new(self.distance, self.phi, self.theta).to_vec3();
        Transform::looking_at(self.target + offset, self.target, Vec3::Y)
    }

    /// Applies the controller pose to a perspective camera.
    pub fn apply_to_perspective(&self, camera: &mut PerspectiveCamera) {
        let transform = self.camera_transform();
        camera.position = transform.translation;
        camera.target = self.target;
        camera.up = Vec3::Y;
    }

    fn clamp_state(&mut self) {
        self.min_distance = self.min_distance.max(0.001);
        self.max_distance = self.max_distance.max(self.min_distance);
        self.distance = clamp(self.distance, self.min_distance, self.max_distance);
        self.min_polar_angle = clamp(self.min_polar_angle, 0.0, core::f32::consts::PI);
        self.max_polar_angle = clamp(
            self.max_polar_angle,
            self.min_polar_angle,
            core::f32::consts::PI,
        );
        self.phi = clamp(self.phi, self.min_polar_angle, self.max_polar_angle);
    }
}

impl Default for OrbitController {
    #[inline]
    fn default() -> Self {
        Self::new(Vec3::ZERO, 5.0)
    }
}

impl FlyController {
    /// Creates a fly controller at a position.
    #[inline]
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
            speed: 5.0,
            fast_multiplier: 4.0,
            sensitivity: 0.003,
            pitch_limit: core::f32::consts::FRAC_PI_2 - 0.001,
        }
    }

    /// Consumes keyboard and pointer state, then returns the camera transform.
    pub fn update(&mut self, keyboard: KeyboardState, pointer: PointerState, dt: f32) -> Transform {
        self.yaw -= pointer.delta.x * self.sensitivity;
        self.pitch -= pointer.delta.y * self.sensitivity;
        self.pitch = clamp(self.pitch, -self.pitch_limit, self.pitch_limit);

        let rotation = self.rotation();
        let forward = rotation.mul_vec3(Vec3::NEG_Z).normalize();
        let right = rotation.mul_vec3(Vec3::X).normalize();
        let up = Vec3::Y;
        let mut movement = Vec3::ZERO;

        if keyboard.is_pressed(KeyCode::KeyW) || keyboard.is_pressed(KeyCode::ArrowUp) {
            movement += forward;
        }
        if keyboard.is_pressed(KeyCode::KeyS) || keyboard.is_pressed(KeyCode::ArrowDown) {
            movement -= forward;
        }
        if keyboard.is_pressed(KeyCode::KeyD) || keyboard.is_pressed(KeyCode::ArrowRight) {
            movement += right;
        }
        if keyboard.is_pressed(KeyCode::KeyA) || keyboard.is_pressed(KeyCode::ArrowLeft) {
            movement -= right;
        }
        if keyboard.is_pressed(KeyCode::KeyE) || keyboard.is_pressed(KeyCode::Space) {
            movement += up;
        }
        if keyboard.is_pressed(KeyCode::KeyQ) {
            movement -= up;
        }

        if movement.length_squared() > crate::EPSILON {
            let mut speed = self.speed;
            if keyboard.modifiers().shift {
                speed *= self.fast_multiplier.max(1.0);
            }
            self.position += movement.normalize() * speed * dt.max(0.0);
        }

        Transform::new(self.position, rotation, Vec3::ONE)
    }

    /// Returns the current rotation.
    #[inline]
    pub fn rotation(&self) -> Quat {
        Quat::from_axis_angle(Vec3::Y, self.yaw) * Quat::from_axis_angle(Vec3::X, self.pitch)
    }

    /// Applies the controller pose to a perspective camera.
    pub fn apply_to_perspective(&self, camera: &mut PerspectiveCamera) {
        let rotation = self.rotation();
        let forward = rotation.mul_vec3(Vec3::NEG_Z).normalize();
        camera.position = self.position;
        camera.target = self.position + forward;
        camera.up = rotation.mul_vec3(Vec3::Y).normalize();
    }
}

impl Default for FlyController {
    #[inline]
    fn default() -> Self {
        Self::new(Vec3::ZERO)
    }
}
