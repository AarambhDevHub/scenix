use core::fmt;
use core::hash::{Hash, Hasher};

use scenix_math::{Vec2, Vec3, Vec4};

/// Floating-point node value types produced by shader nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeValueKind {
    /// 32-bit float.
    #[default]
    Float,
    /// Two-component vector.
    Vec2,
    /// Three-component vector.
    Vec3,
    /// Four-component vector.
    Vec4,
}

impl NodeValueKind {
    /// Returns the WGSL type name.
    #[inline]
    pub const fn wgsl_type(self) -> &'static str {
        match self {
            Self::Float => "f32",
            Self::Vec2 => "vec2<f32>",
            Self::Vec3 => "vec3<f32>",
            Self::Vec4 => "vec4<f32>",
        }
    }

    /// Returns the GLSL ES type name.
    #[inline]
    pub const fn glsl_type(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::Vec2 => "vec2",
            Self::Vec3 => "vec3",
            Self::Vec4 => "vec4",
        }
    }

    /// Returns the number of scalar components.
    #[inline]
    pub const fn components(self) -> usize {
        match self {
            Self::Float => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
        }
    }

    /// Returns whether this type is a scalar.
    #[inline]
    pub const fn is_float(self) -> bool {
        matches!(self, Self::Float)
    }
}

impl fmt::Display for NodeValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.wgsl_type())
    }
}

/// Inline constant value carried directly by a node or node input.
///
/// Float payloads are compared and hashed by their bit patterns so identical
/// constants (including `NaN` and signed zero) merge deterministically in
/// common-subexpression elimination.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeValue {
    /// Scalar constant.
    Float(f32),
    /// Two-component constant.
    Vec2(Vec2),
    /// Three-component constant.
    Vec3(Vec3),
    /// Four-component constant.
    Vec4(Vec4),
}

impl PartialEq for NodeValue {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.bit_pattern() == other.bit_pattern()
    }
}

impl Eq for NodeValue {}

impl Hash for NodeValue {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bit_pattern().hash(state);
    }
}

impl NodeValue {
    /// Creates a scalar constant.
    #[inline]
    pub const fn float(value: f32) -> Self {
        Self::Float(value)
    }

    /// Creates a three-component constant from linear RGB components.
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::Vec3(Vec3::new(r, g, b))
    }

    /// Creates a four-component constant.
    #[inline]
    pub const fn vec4(values: Vec4) -> Self {
        Self::Vec4(values)
    }

    /// Creates an all-`value` constant of `kind`.
    #[inline]
    pub fn splat(kind: NodeValueKind, value: f32) -> Self {
        match kind {
            NodeValueKind::Float => Self::Float(value),
            NodeValueKind::Vec2 => Self::Vec2(Vec2::new(value, value)),
            NodeValueKind::Vec3 => Self::Vec3(Vec3::new(value, value, value)),
            NodeValueKind::Vec4 => Self::Vec4(Vec4::new(value, value, value, value)),
        }
    }

    /// Returns the value type.
    #[inline]
    pub const fn kind(&self) -> NodeValueKind {
        match self {
            Self::Float(_) => NodeValueKind::Float,
            Self::Vec2(_) => NodeValueKind::Vec2,
            Self::Vec3(_) => NodeValueKind::Vec3,
            Self::Vec4(_) => NodeValueKind::Vec4,
        }
    }

    /// Returns the bit pattern used for equality and hashing.
    #[inline]
    pub(crate) const fn bit_pattern(&self) -> [u32; 4] {
        match self {
            Self::Float(value) => [value.to_bits(), 0, 0, 0],
            Self::Vec2(value) => [value.x.to_bits(), value.y.to_bits(), 0, 0],
            Self::Vec3(value) => [value.x.to_bits(), value.y.to_bits(), value.z.to_bits(), 0],
            Self::Vec4(value) => [
                value.x.to_bits(),
                value.y.to_bits(),
                value.z.to_bits(),
                value.w.to_bits(),
            ],
        }
    }

    /// Returns the scalar at `index`, or `None` past the component count.
    #[inline]
    pub(crate) fn component(&self, index: usize) -> Option<f32> {
        match self {
            Self::Float(value) if index == 0 => Some(*value),
            Self::Vec2(value) => match index {
                0 => Some(value.x),
                1 => Some(value.y),
                _ => None,
            },
            Self::Vec3(value) => match index {
                0 => Some(value.x),
                1 => Some(value.y),
                2 => Some(value.z),
                _ => None,
            },
            Self::Vec4(value) => match index {
                0 => Some(value.x),
                1 => Some(value.y),
                2 => Some(value.z),
                3 => Some(value.w),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns a copy with the scalar broadcast into `kind`'s components.
    #[inline]
    pub fn broadcast(self, kind: NodeValueKind) -> Self {
        if self.kind() == kind {
            return self;
        }
        if matches!(self, Self::Float(_)) {
            let Self::Float(value) = self else {
                unreachable!("matched Float above")
            };
            return Self::splat(kind, value);
        }
        let first = self.component(0).unwrap_or(0.0);
        Self::splat(kind, first)
    }
}

impl Default for NodeValue {
    #[inline]
    fn default() -> Self {
        Self::Float(0.0)
    }
}
