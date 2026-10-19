//! Environment capability notes for the browser fallback renderers
//! (scenix v1.8.0).
//!
//! The `scenix-renderer` `environment` feature (terrain splats, sky domes,
//! water surfaces) drives wgpu (WebGPU in the browser). Every scenix browser
//! fallback also needs a stated environment plan so WASM targets degrade
//! predictably:
//!
//! * **WebGPU** — full parity: the three WGSL pipelines plus the
//!   reflection/refraction render-target reads for water all compile.
//! * **WebGL 2** — framebuffers and float textures are real, so
//!   reflection/refraction stay available, but the dynamic uniform-offset
//!   belt is unavailable; hosts translate each environment material into a
//!   static set of GLSL uniforms and re-upload per slot.
//! * **WebGL 1** — render-to-texture float targets and instancing are not
//!   guaranteed; the fallback tints expanded terrain/water geometry with
//!   CPU-baked splat weights on the vertex color attribute, drives the sky
//!   dome from vertex colors alone, and drops screen-space water effects.
//!
//! [`environment_fallback_plan`] turns a [`WebGlCapabilityLevel`] into this
//! decision so callers can branch once instead of string-matching labels.

use crate::WebGlCapabilityLevel;

/// How environment materials are rendered by a browser fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvironmentFallbackPlan {
    /// WGSL pipelines + reflection/refraction targets (WebGPU).
    Full,
    /// Static per-slot GLSL uniforms; screen-space water survives.
    StaticUniforms,
    /// CPU-baked vertex-color tinting; no screen-space water effects.
    VertexColorTint,
}

impl EnvironmentFallbackPlan {
    /// Returns whether water can read reflection/refraction render targets.
    #[inline]
    pub const fn supports_screen_space_water(self) -> bool {
        match self {
            Self::Full => true,
            Self::StaticUniforms => true,
            Self::VertexColorTint => false,
        }
    }

    /// Returns whether the CPU must bake splat weights into vertex colors.
    #[inline]
    pub const fn bakes_splat_weights(self) -> bool {
        matches!(self, Self::VertexColorTint)
    }

    /// Compact diagnostics label.
    #[inline]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Full => "environment-full",
            Self::StaticUniforms => "environment-static-uniforms",
            Self::VertexColorTint => "environment-vertex-color",
        }
    }
}

/// Resolves the environment plan available at a browser capability level.
#[inline]
pub const fn environment_fallback_plan(level: WebGlCapabilityLevel) -> EnvironmentFallbackPlan {
    match level {
        WebGlCapabilityLevel::WebGl1 => EnvironmentFallbackPlan::VertexColorTint,
        WebGlCapabilityLevel::WebGl2 => EnvironmentFallbackPlan::StaticUniforms,
    }
}

#[cfg(test)]
mod tests {
    use super::{EnvironmentFallbackPlan as Plan, environment_fallback_plan};
    use crate::WebGlCapabilityLevel;

    #[test]
    fn webgl1_tints_vertices_and_webgl2_uses_static_uniforms() {
        let webgl1 = environment_fallback_plan(WebGlCapabilityLevel::WebGl1);
        assert_eq!(webgl1, Plan::VertexColorTint);
        assert!(webgl1.bakes_splat_weights());
        assert!(!webgl1.supports_screen_space_water());
        assert_eq!(webgl1.label(), "environment-vertex-color");

        let webgl2 = environment_fallback_plan(WebGlCapabilityLevel::WebGl2);
        assert_eq!(webgl2, Plan::StaticUniforms);
        assert!(!webgl2.bakes_splat_weights());
        assert!(webgl2.supports_screen_space_water());
        assert_eq!(webgl2.label(), "environment-static-uniforms");

        let full = Plan::Full;
        assert!(full.supports_screen_space_water());
        assert!(!full.bakes_splat_weights());
        assert_eq!(full.label(), "environment-full");
    }
}
