//! Particle capability notes for the browser fallback renderers
//! (scenix v1.7.0).
//!
//! The `scenix-renderer` particle feature drives wgpu (WebGPU in the
//! browser). Every scenix browser fallback also needs a stated particle
//! plan so WASM targets degrade predictably:
//!
//! * **WebGPU** — full parity: the billboard draw path plus the optional
//!   48-byte pool compute simulation both compile.
//! * **WebGL 2** — instanced billboard quads are supported through core
//!   instanced-array attributes, but the compute path is unavailable; hosts
//!   keep `scenix_particles::ParticleSystem` on the CPU and upload batches
//!   per frame.
//! * **WebGL 1** — instanced attributes only exist behind
//!   `ANGLE_instanced_arrays`, which not every WebGL 1 context exposes;
//!   scenix's WebGL 1 fallback therefore expands billboards into real quad
//!   geometry (`scenix_mesh::quad_geometry`) and re-draws per particle.
//!
//! [`particle_fallback_plan`] turns a [`WebGlCapabilityLevel`] into this
//! decision so callers can branch once instead of string-matching labels.

use crate::WebGlCapabilityLevel;

/// How one particle system is rendered by a browser fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticleFallbackPlan {
    /// Billboard draw path + optional compute simulation (WebGPU).
    Full,
    /// Instanced billboard quads; particles simulate on the CPU.
    InstancedBillboards,
    /// Per-particle quad geometry re-draw; the widest-compatibility path.
    ExpandedQuads,
}

impl ParticleFallbackPlan {
    /// Returns whether compute simulation runs on this plan.
    #[inline]
    pub const fn supports_compute(self) -> bool {
        matches!(self, Self::Full)
    }

    /// Compact diagnostics label.
    #[inline]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Full => "particles-full",
            Self::InstancedBillboards => "particles-instanced",
            Self::ExpandedQuads => "particles-expanded",
        }
    }
}

/// Resolves the particle plan available at a browser capability level.
#[inline]
pub const fn particle_fallback_plan(level: WebGlCapabilityLevel) -> ParticleFallbackPlan {
    match level {
        WebGlCapabilityLevel::WebGl1 => ParticleFallbackPlan::ExpandedQuads,
        WebGlCapabilityLevel::WebGl2 => ParticleFallbackPlan::InstancedBillboards,
    }
}

#[cfg(test)]
mod tests {
    use super::{ParticleFallbackPlan, particle_fallback_plan};
    use crate::WebGlCapabilityLevel;

    #[test]
    fn webgl1_expands_quads_and_webgl2_instances() {
        assert_eq!(
            particle_fallback_plan(WebGlCapabilityLevel::WebGl1),
            ParticleFallbackPlan::ExpandedQuads
        );
        assert!(!particle_fallback_plan(WebGlCapabilityLevel::WebGl1).supports_compute());

        assert_eq!(
            particle_fallback_plan(WebGlCapabilityLevel::WebGl2),
            ParticleFallbackPlan::InstancedBillboards
        );
        assert!(!particle_fallback_plan(WebGlCapabilityLevel::WebGl2).supports_compute());
    }
}
