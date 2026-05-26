use core::fmt;

/// Top-level scenix error type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScenixError {
    /// Asset loading failed.
    Load(LoadError),
    /// GPU setup or upload failed.
    Gpu(GpuError),
    /// Validation failed.
    Validation(ValidationError),
}

/// Asset loading errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadError {
    /// The input format is unsupported.
    UnsupportedFormat,
    /// The source uses a known format feature that scenix does not support yet.
    UnsupportedFeature,
    /// The input bytes or text could not be parsed.
    Parse,
    /// The input could not be decoded.
    Decode,
    /// An IO or network request failed.
    Io,
    /// The requested asset was not found.
    NotFound,
}

/// GPU-related errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GpuError {
    /// GPU backend initialization failed.
    Init,
    /// GPU resource upload failed.
    Upload,
    /// GPU feature or limit is unsupported.
    Unsupported,
}

/// Validation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValidationError {
    /// An ID did not refer to a known object.
    InvalidId,
    /// A numeric value was outside the accepted range.
    OutOfRange,
    /// A value combination is invalid.
    InvalidState,
}

impl fmt::Display for ScenixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(err) => write!(f, "load error: {err}"),
            Self::Gpu(err) => write!(f, "gpu error: {err}"),
            Self::Validation(err) => write!(f, "validation error: {err}"),
        }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat => f.write_str("unsupported format"),
            Self::UnsupportedFeature => f.write_str("unsupported asset feature"),
            Self::Parse => f.write_str("parse failed"),
            Self::Decode => f.write_str("decode failed"),
            Self::Io => f.write_str("io failed"),
            Self::NotFound => f.write_str("asset not found"),
        }
    }
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init => f.write_str("initialization failed"),
            Self::Upload => f.write_str("resource upload failed"),
            Self::Unsupported => f.write_str("unsupported gpu feature"),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId => f.write_str("invalid id"),
            Self::OutOfRange => f.write_str("value out of range"),
            Self::InvalidState => f.write_str("invalid state"),
        }
    }
}

impl From<LoadError> for ScenixError {
    #[inline]
    fn from(value: LoadError) -> Self {
        Self::Load(value)
    }
}

impl From<GpuError> for ScenixError {
    #[inline]
    fn from(value: GpuError) -> Self {
        Self::Gpu(value)
    }
}

impl From<ValidationError> for ScenixError {
    #[inline]
    fn from(value: ValidationError) -> Self {
        Self::Validation(value)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ScenixError {}
#[cfg(feature = "std")]
impl std::error::Error for LoadError {}
#[cfg(feature = "std")]
impl std::error::Error for GpuError {}
#[cfg(feature = "std")]
impl std::error::Error for ValidationError {}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn errors_display_without_allocation_payloads() {
        assert_eq!(
            ScenixError::from(ValidationError::InvalidId).to_string(),
            "validation error: invalid id"
        );
    }
}
