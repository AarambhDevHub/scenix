use animato::{Easing, Spring, SpringConfig, Tween, Update};
use scenix_core::Color;
use scenix_math::{Quat, Vec3};

use crate::{AnimColor, AnimQuat, AnimVec3};

/// A scalar animation track backed by Animato tween or spring primitives.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScalarTrack {
    /// Finite tween from start to end.
    Tween(Tween<f32>),
    /// Physics spring toward a target.
    Spring(Spring),
}

impl ScalarTrack {
    /// Creates a linear scalar tween.
    #[inline]
    pub fn tween(start: f32, end: f32, duration: f32) -> Self {
        Self::tween_with_easing(start, end, duration, Easing::Linear)
    }

    /// Creates a scalar tween with an easing curve.
    #[inline]
    pub fn tween_with_easing(start: f32, end: f32, duration: f32, easing: Easing) -> Self {
        Self::Tween(
            Tween::new(start, end)
                .duration(duration)
                .easing(easing)
                .build(),
        )
    }

    /// Creates a scalar spring initialized at `start` and moving toward `target`.
    #[inline]
    pub fn spring(start: f32, target: f32, config: SpringConfig) -> Self {
        Self::Spring(spring1(start, target, config))
    }

    /// Advances the track and returns whether it is still running.
    #[inline]
    pub fn update(&mut self, dt: f32) -> bool {
        match self {
            Self::Tween(track) => track.update(dt),
            Self::Spring(track) => track.update(dt),
        }
    }

    /// Returns the current scalar value.
    #[inline]
    pub fn value(&self) -> f32 {
        match self {
            Self::Tween(track) => track.value(),
            Self::Spring(track) => track.position(),
        }
    }

    /// Returns whether the track has completed or settled.
    #[inline]
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Tween(track) => track.is_complete(),
            Self::Spring(track) => track.is_settled(),
        }
    }
}

/// A 3D vector animation track.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Vec3Track {
    /// Finite tween using Animato interpolation.
    Tween(Tween<AnimVec3>),
    /// Component-wise Animato springs.
    Spring {
        /// X-axis spring.
        x: Spring,
        /// Y-axis spring.
        y: Spring,
        /// Z-axis spring.
        z: Spring,
    },
}

impl Vec3Track {
    /// Creates a linear vector tween.
    #[inline]
    pub fn tween(start: Vec3, end: Vec3, duration: f32) -> Self {
        Self::tween_with_easing(start, end, duration, Easing::Linear)
    }

    /// Creates a vector tween with an easing curve.
    #[inline]
    pub fn tween_with_easing(start: Vec3, end: Vec3, duration: f32, easing: Easing) -> Self {
        Self::Tween(
            Tween::new(AnimVec3(start), AnimVec3(end))
                .duration(duration)
                .easing(easing)
                .build(),
        )
    }

    /// Creates component-wise springs initialized at `start` and moving toward `target`.
    #[inline]
    pub fn spring(start: Vec3, target: Vec3, config: SpringConfig) -> Self {
        Self::Spring {
            x: spring1(start.x, target.x, config.clone()),
            y: spring1(start.y, target.y, config.clone()),
            z: spring1(start.z, target.z, config),
        }
    }

    /// Advances the track and returns whether it is still running.
    #[inline]
    pub fn update(&mut self, dt: f32) -> bool {
        match self {
            Self::Tween(track) => track.update(dt),
            Self::Spring { x, y, z } => x.update(dt) | y.update(dt) | z.update(dt),
        }
    }

    /// Returns the current vector value.
    #[inline]
    pub fn value(&self) -> Vec3 {
        match self {
            Self::Tween(track) => track.value().0,
            Self::Spring { x, y, z } => Vec3::new(x.position(), y.position(), z.position()),
        }
    }

    /// Returns whether the track has completed or settled.
    #[inline]
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Tween(track) => track.is_complete(),
            Self::Spring { x, y, z } => x.is_settled() && y.is_settled() && z.is_settled(),
        }
    }
}

/// A quaternion rotation animation track.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QuatTrack {
    /// Finite tween using quaternion slerp.
    Tween(Tween<AnimQuat>),
}

impl QuatTrack {
    /// Creates a quaternion tween.
    #[inline]
    pub fn tween(start: Quat, end: Quat, duration: f32) -> Self {
        Self::tween_with_easing(start, end, duration, Easing::Linear)
    }

    /// Creates a quaternion tween with an easing curve.
    #[inline]
    pub fn tween_with_easing(start: Quat, end: Quat, duration: f32, easing: Easing) -> Self {
        Self::Tween(
            Tween::new(AnimQuat(start), AnimQuat(end))
                .duration(duration)
                .easing(easing)
                .build(),
        )
    }

    /// Advances the track and returns whether it is still running.
    #[inline]
    pub fn update(&mut self, dt: f32) -> bool {
        match self {
            Self::Tween(track) => track.update(dt),
        }
    }

    /// Returns the current normalized quaternion.
    #[inline]
    pub fn value(&self) -> Quat {
        match self {
            Self::Tween(track) => track.value().0.normalize(),
        }
    }

    /// Returns whether the track has completed.
    #[inline]
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Tween(track) => track.is_complete(),
        }
    }
}

/// A color animation track.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ColorTrack {
    /// Finite tween using color-channel interpolation.
    Tween(Tween<AnimColor>),
    /// Component-wise Animato springs.
    Spring {
        /// Red channel spring.
        r: Spring,
        /// Green channel spring.
        g: Spring,
        /// Blue channel spring.
        b: Spring,
        /// Alpha channel spring.
        a: Spring,
    },
}

impl ColorTrack {
    /// Creates a linear color tween.
    #[inline]
    pub fn tween(start: Color, end: Color, duration: f32) -> Self {
        Self::tween_with_easing(start, end, duration, Easing::Linear)
    }

    /// Creates a color tween with an easing curve.
    #[inline]
    pub fn tween_with_easing(start: Color, end: Color, duration: f32, easing: Easing) -> Self {
        Self::Tween(
            Tween::new(AnimColor(start), AnimColor(end))
                .duration(duration)
                .easing(easing)
                .build(),
        )
    }

    /// Creates component-wise springs initialized at `start` and moving toward `target`.
    #[inline]
    pub fn spring(start: Color, target: Color, config: SpringConfig) -> Self {
        Self::Spring {
            r: spring1(start.r, target.r, config.clone()),
            g: spring1(start.g, target.g, config.clone()),
            b: spring1(start.b, target.b, config.clone()),
            a: spring1(start.a, target.a, config),
        }
    }

    /// Advances the track and returns whether it is still running.
    #[inline]
    pub fn update(&mut self, dt: f32) -> bool {
        match self {
            Self::Tween(track) => track.update(dt),
            Self::Spring { r, g, b, a } => {
                r.update(dt) | g.update(dt) | b.update(dt) | a.update(dt)
            }
        }
    }

    /// Returns the current color value.
    #[inline]
    pub fn value(&self) -> Color {
        match self {
            Self::Tween(track) => track.value().0,
            Self::Spring { r, g, b, a } => {
                Color::rgba(r.position(), g.position(), b.position(), a.position())
            }
        }
    }

    /// Returns whether the track has completed or settled.
    #[inline]
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Tween(track) => track.is_complete(),
            Self::Spring { r, g, b, a } => {
                r.is_settled() && g.is_settled() && b.is_settled() && a.is_settled()
            }
        }
    }
}

/// A time-delayed boolean switch track.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoolTrack {
    start: bool,
    end: bool,
    duration: f32,
    elapsed: f32,
    value: bool,
}

impl BoolTrack {
    /// Creates a boolean track that switches at the end of `duration`.
    #[inline]
    pub fn new(start: bool, end: bool, duration: f32) -> Self {
        Self {
            start,
            end,
            duration: duration.max(0.0),
            elapsed: 0.0,
            value: start,
        }
    }

    /// Creates an immediate boolean value.
    #[inline]
    pub const fn immediate(value: bool) -> Self {
        Self {
            start: value,
            end: value,
            duration: 0.0,
            elapsed: 0.0,
            value,
        }
    }

    /// Advances the track and returns whether it is still running.
    #[inline]
    pub fn update(&mut self, dt: f32) -> bool {
        if self.is_complete() {
            self.value = self.end;
            return false;
        }
        self.elapsed = (self.elapsed + dt.max(0.0)).min(self.duration);
        if self.is_complete() {
            self.value = self.end;
            false
        } else {
            self.value = self.start;
            true
        }
    }

    /// Returns the current boolean value.
    #[inline]
    pub const fn value(&self) -> bool {
        self.value
    }

    /// Returns whether the switch is complete.
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.duration == 0.0 || self.elapsed >= self.duration
    }
}

fn spring1(start: f32, target: f32, config: SpringConfig) -> Spring {
    let mut spring = Spring::new(config);
    spring.snap_to(start);
    spring.set_target(target);
    spring
}
