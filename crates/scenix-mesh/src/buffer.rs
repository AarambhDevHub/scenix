use alloc::vec::Vec;

/// Semantic meaning of a vertex attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VertexSemantic {
    /// Vertex position.
    Position,
    /// Vertex normal.
    Normal,
    /// Primary UV coordinate.
    Uv0,
    /// Secondary UV coordinate.
    Uv1,
    /// Vertex color.
    Color,
    /// Tangent with handedness.
    Tangent,
    /// Per-instance transform matrix.
    InstanceMatrix,
}

/// Plain vertex data format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VertexFormat {
    /// Two 32-bit floats.
    Float32x2,
    /// Three 32-bit floats.
    Float32x3,
    /// Four 32-bit floats.
    Float32x4,
    /// One 32-bit unsigned integer.
    Uint32,
}

/// Index buffer integer format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IndexFormat {
    /// 16-bit unsigned integer indices.
    Uint16,
    /// 32-bit unsigned integer indices.
    Uint32,
}

/// Whether a vertex buffer advances per vertex or per instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BufferStepMode {
    /// Advance once per vertex.
    Vertex,
    /// Advance once per instance.
    Instance,
}

/// A single vertex attribute in an interleaved or packed buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VertexAttribute {
    /// Attribute semantic.
    pub semantic: VertexSemantic,
    /// Attribute storage format.
    pub format: VertexFormat,
    /// Byte offset from the start of the vertex.
    pub offset: u64,
    /// Shader location for renderer backends.
    pub shader_location: u32,
}

impl VertexAttribute {
    /// Creates a vertex attribute descriptor.
    #[inline]
    pub const fn new(
        semantic: VertexSemantic,
        format: VertexFormat,
        offset: u64,
        shader_location: u32,
    ) -> Self {
        Self {
            semantic,
            format,
            offset,
            shader_location,
        }
    }
}

/// Vertex buffer layout metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BufferLayout {
    /// Byte stride between elements.
    pub array_stride: u64,
    /// Step mode for the buffer.
    pub step_mode: BufferStepMode,
    /// Attributes stored in the buffer.
    pub attributes: Vec<VertexAttribute>,
}

impl BufferLayout {
    /// Creates a buffer layout.
    #[inline]
    pub fn new(
        array_stride: u64,
        step_mode: BufferStepMode,
        attributes: Vec<VertexAttribute>,
    ) -> Self {
        Self {
            array_stride,
            step_mode,
            attributes,
        }
    }
}
