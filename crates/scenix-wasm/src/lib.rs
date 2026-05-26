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

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::{WebRenderer, canvas_size};

#[cfg(not(target_arch = "wasm32"))]
/// Browser renderer wrapper.
///
/// The concrete implementation is available when compiling for
/// `wasm32-unknown-unknown`.
#[derive(Debug)]
pub struct WebRenderer;
