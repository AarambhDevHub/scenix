//! Browser/WASM integration helpers for scenix.

mod input;

pub use input::{key_code_from_dom, pointer_button_from_dom};

/// Installs a panic hook that forwards Rust panics to the browser console.
#[inline]
pub fn set_panic_hook() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

/// Clamps a canvas/render target size to renderer-valid dimensions.
#[inline]
pub const fn clamp_canvas_size(width: u32, height: u32) -> (u32, u32) {
    (
        if width == 0 { 1 } else { width },
        if height == 0 { 1 } else { height },
    )
}

/// WebGL capability level used by the browser fallback renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebGlCapabilityLevel {
    /// WebGL 1 reduced fallback path.
    WebGl1,
    /// WebGL 2 full browser fallback path for the generated renderer scene.
    WebGl2,
}

impl WebGlCapabilityLevel {
    /// Returns a compact label used in diagnostics.
    #[inline]
    pub const fn label(self) -> &'static str {
        match self {
            Self::WebGl1 => "webgl1",
            Self::WebGl2 => "webgl2",
        }
    }

    /// Returns the renderer parity level for this browser fallback.
    #[inline]
    pub const fn parity_label(self) -> &'static str {
        match self {
            Self::WebGl1 => "reduced-fallback",
            Self::WebGl2 => "full-fallback",
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::{
    BrowserBackendKind, BrowserBackendPreference, BrowserRenderer, WebGlRenderer, WebRenderer,
    canvas_size,
};

#[cfg(not(target_arch = "wasm32"))]
/// Browser renderer wrapper.
///
/// The concrete implementation is available when compiling for
/// `wasm32-unknown-unknown`.
#[derive(Debug)]
pub struct WebRenderer;

#[cfg(not(target_arch = "wasm32"))]
/// Browser renderer with automatic WebGPU/WebGL backend selection.
#[derive(Debug)]
pub struct BrowserRenderer;

#[cfg(not(target_arch = "wasm32"))]
/// Browser WebGL fallback renderer.
#[derive(Debug)]
pub struct WebGlRenderer;

#[cfg(not(target_arch = "wasm32"))]
/// Preferred browser rendering backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserBackendPreference {
    /// Select the best available browser backend.
    Auto,
    /// Force WebGPU.
    WebGpu,
    /// Force WebGL.
    WebGl,
}

#[cfg(not(target_arch = "wasm32"))]
/// Active browser rendering backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserBackendKind {
    /// WebGPU backend.
    WebGpu,
    /// WebGL backend.
    WebGl,
    /// Application-level Canvas2D fallback.
    CanvasFallback,
}
