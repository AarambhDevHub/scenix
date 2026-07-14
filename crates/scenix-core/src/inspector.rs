use alloc::{string::String, vec::Vec};

use scenix_math::{Vec2, Vec3, Vec4};

use crate::Color;

/// Opaque identifier used to correlate inspector rows with application data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InspectorId(pub u64);

/// A typed inspector field value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InspectorValue {
    /// Boolean value.
    Bool(bool),
    /// Signed integer value.
    Integer(i64),
    /// Unsigned integer value.
    Unsigned(u64),
    /// Floating-point value.
    Number(f64),
    /// Human-readable text or enum label.
    Text(String),
    /// Two-dimensional vector.
    Vec2(Vec2),
    /// Three-dimensional vector.
    Vec3(Vec3),
    /// Four-dimensional vector.
    Vec4(Vec4),
    /// Linear color.
    Color(Color),
    /// Byte count rendered with resource-aware formatting.
    Bytes(u64),
}

impl From<bool> for InspectorValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<f32> for InspectorValue {
    fn from(value: f32) -> Self {
        Self::Number(value as f64)
    }
}
impl From<f64> for InspectorValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}
impl From<u64> for InspectorValue {
    fn from(value: u64) -> Self {
        Self::Unsigned(value)
    }
}
impl From<usize> for InspectorValue {
    fn from(value: usize) -> Self {
        Self::Unsigned(value as u64)
    }
}
impl From<Vec2> for InspectorValue {
    fn from(value: Vec2) -> Self {
        Self::Vec2(value)
    }
}
impl From<Vec3> for InspectorValue {
    fn from(value: Vec3) -> Self {
        Self::Vec3(value)
    }
}
impl From<Vec4> for InspectorValue {
    fn from(value: Vec4) -> Self {
        Self::Vec4(value)
    }
}
impl From<Color> for InspectorValue {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}
impl From<String> for InspectorValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}
impl From<&str> for InspectorValue {
    fn from(value: &str) -> Self {
        Self::Text(value.into())
    }
}

/// One named property in an inspector item.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InspectorField {
    /// Field label.
    pub name: String,
    /// Typed field value.
    pub value: InspectorValue,
}

impl InspectorField {
    /// Creates a named field.
    pub fn new(name: impl Into<String>, value: impl Into<InspectorValue>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Hierarchical inspector row with fields and children.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InspectorItem {
    /// Stable snapshot-local item identifier.
    pub id: InspectorId,
    /// Display label.
    pub label: String,
    /// Short type/category label.
    pub kind: String,
    /// Read-only typed fields.
    pub fields: Vec<InspectorField>,
    /// Nested items.
    pub children: Vec<InspectorItem>,
}

impl InspectorItem {
    /// Creates an empty inspector item.
    pub fn new(id: InspectorId, label: impl Into<String>, kind: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            kind: kind.into(),
            fields: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Appends a typed field.
    pub fn field(mut self, name: impl Into<String>, value: impl Into<InspectorValue>) -> Self {
        self.fields.push(InspectorField::new(name, value));
        self
    }

    /// Appends a child item.
    pub fn child(mut self, child: InspectorItem) -> Self {
        self.children.push(child);
        self
    }
}

/// Owned, serialization-friendly inspector tree.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InspectorSnapshot {
    /// Snapshot roots.
    pub roots: Vec<InspectorItem>,
}

impl InspectorSnapshot {
    /// Creates an empty snapshot.
    pub const fn new() -> Self {
        Self { roots: Vec::new() }
    }

    /// Clears content while retaining allocated capacity.
    pub fn clear(&mut self) {
        self.roots.clear();
    }

    /// Appends a root item.
    pub fn push(&mut self, item: InspectorItem) {
        self.roots.push(item);
    }
}

/// Appends a structured view of a value to an inspector snapshot.
pub trait Inspectable {
    /// Appends one or more roots without clearing existing content.
    fn inspect(&self, snapshot: &mut InspectorSnapshot);

    /// Builds a new owned snapshot.
    fn inspector_snapshot(&self) -> InspectorSnapshot {
        let mut snapshot = InspectorSnapshot::new();
        self.inspect(&mut snapshot);
        snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_tree_builders_preserve_typed_values() {
        let item = InspectorItem::new(InspectorId(1), "Camera", "perspective")
            .field("fov", 60.0_f32)
            .child(InspectorItem::new(InspectorId(2), "Transform", "transform"));
        assert_eq!(item.children.len(), 1);
        assert_eq!(item.fields[0].value, InspectorValue::Number(60.0));
    }
}
