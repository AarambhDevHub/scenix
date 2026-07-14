//! Browser/WASM integration helpers for scenix.

mod input;

pub use input::{
    gamepad_axis_from_standard, gamepad_button_from_standard, key_code_from_dom,
    pointer_button_from_dom, touch_phase_from_dom,
};

/// Device-pixel-ratio-aware browser canvas measurements.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasMetrics {
    /// CSS width in logical pixels.
    pub logical_width: u32,
    /// CSS height in logical pixels.
    pub logical_height: u32,
    /// Backing-buffer width in physical pixels.
    pub physical_width: u32,
    /// Backing-buffer height in physical pixels.
    pub physical_height: u32,
    /// Physical pixels per logical pixel.
    pub device_pixel_ratio: f32,
}

impl CanvasMetrics {
    /// Creates sanitized metrics from CSS dimensions and DPR.
    pub fn new(logical_width: u32, logical_height: u32, device_pixel_ratio: f32) -> Self {
        let (logical_width, logical_height) = clamp_canvas_size(logical_width, logical_height);
        let device_pixel_ratio = if device_pixel_ratio.is_finite() && device_pixel_ratio > 0.0 {
            device_pixel_ratio
        } else {
            1.0
        };
        Self {
            logical_width,
            logical_height,
            physical_width: (logical_width as f32 * device_pixel_ratio).round().max(1.0) as u32,
            physical_height: (logical_height as f32 * device_pixel_ratio)
                .round()
                .max(1.0) as u32,
            device_pixel_ratio,
        }
    }
}

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
    canvas_metrics, canvas_size,
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
