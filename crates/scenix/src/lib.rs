#![cfg_attr(not(feature = "std"), no_std)]

//! Facade crate for scenix Renderer APIs.
//!
//! This release re-exports the Foundation crates, the GPU-free scene graph,
//! CPU-side geometry, materials, lights, textures, cameras, optional loaders,
//! raycasting, debug helper geometry, optional Animato and WASM integration,
//! optional post-processing, and optional renderer APIs.

extern crate alloc;

pub use scenix_core::*;
pub use scenix_input::*;
pub use scenix_math::*;

#[cfg(feature = "scene")]
pub use scenix_scene::*;

#[cfg(feature = "camera")]
pub use scenix_camera::*;

#[cfg(feature = "mesh")]
pub use scenix_mesh::*;

#[cfg(feature = "material")]
pub use scenix_material::*;

// v1.8.0 environment crates: GPU-free terrain/sky/water math. The scene
// schema (TerrainId/SkyId/WaterId) and material descriptors arrive via the
// core and material globs above. Terrain/sky lists are curated because the
// `lod`/`sky`/`sun` module names collide with the scene, material, and
// light crate module globs.
#[cfg(feature = "terrain")]
pub use scenix_terrain::{
    ChunkCoord, ChunkStreamer, Heightmap, LodBand, LodPolicy, NoiseConfig, SPLAT_LAYER_COUNT,
    SplatLayer, SplatMap, StreamDiff, TerrainChunk, TerrainCollider, TerrainConfig, TerrainGrid,
    noise_heights,
};

#[cfg(feature = "sky")]
pub use scenix_sky::{
    GroundedSkybox, SkyAmbient, SkyModel, SunPosition, bake_ambient, bake_equirectangular,
    bake_light_probe,
};

#[cfg(feature = "water")]
pub use scenix_water::*;

// v1.9.0 CPU bridges: XR session planning, spatial audio systems, and the
// deterministic physics penalty solver. All three are GPU-free CPU
// source-of-truth crates with no engine or SDK dependency. Curated lists
// beyond v1.8-style globs: the crates' module namespaces (`xr`, `audio`,
// `physics`, `world`, `sync`, ...) and constants would shadow the
// scene/input glob re-exports above, and the camera stereo surface
// (`XrCameraRig`, `XrEyeProjection`, `XrFov`) already arrives via the
// `camera` glob.
#[cfg(feature = "xr")]
pub use scenix_xr::{
    FEATURE_NAMES, JOINT_INDEX_TIP, JOINT_THUMB_TIP, JOINT_WRIST, XR_ANCHOR_MAX_AGE,
    XR_FEATURE_COUNT, XR_HAND_JOINT_COUNT, XR_HIT_MAX_RESULTS, XR_MAX_VIEWPORT_SCALE, XR_VERSION,
    XrAmbientEstimate, XrAnchor, XrAnchorStore, XrController, XrControllerBay,
    XrControllerSnapshot, XrFeatureIter, XrFeatures, XrFoveation, XrFramePlan, XrFrameStats,
    XrHandFrame, XrHit, XrHitSurface, XrHitTestProbe, XrJointPose, XrLightEstimate,
    XrLightEstimationMode, XrPlane, XrPlaneKind, XrPlaneStore, XrPose, XrReferenceSpace,
    XrSessionMode, XrSessionPlan, XrSessionPlanError, XrSessionState, XrSpace, XrViewport,
};

#[cfg(feature = "audio")]
pub use scenix_audio::{
    AUDIO_LISTENER_MAX_SOURCES, AUDIO_VERSION, AnalyserBand, AnalyserFrame, AttenuationModel,
    AudioAnalyser, AudioChannel, AudioListener, AudioLoop, AudioSource, AudioStreamPlan,
    DistanceGain, DopplerModel, DopplerShift, PlaybackState, SPEED_OF_SOUND, STREAM_MAX_CHUNKS,
    STREAM_UNDERRUN_CAPACITY, SpatialDecision, SpatialMix, SpeakerLayout, StreamFormat,
    StreamState, blend_pan, compute_spatial_decision, compute_spatial_gain,
};

#[cfg(feature = "physics")]
pub use scenix_physics::{
    CharacterInput, CharacterState, CharacterStepResult, Collider, ColliderGroup, ColliderShape,
    Contact, ContactEvent, GRAVITY_DEFAULT, GroundContact, MAX_LINEAR_VELOCITY, MAX_SUBSTEP_DT,
    MAX_SUBSTEPS, MIN_BODY_MASS, MIN_FRICTION, PHYSICS_DEFAULT_STEP_OFFSET,
    PHYSICS_GROUND_REATTACH_SPEED, PHYSICS_MAX_CLIMBABLE_SLOPE, PHYSICS_VERSION, PhysicsBody,
    PhysicsTransformSync, PhysicsWorld, REST_CONTACT_VELOCITY, RigidBodyKind,
    SLEEP_FRAME_THRESHOLD, SLEEP_MOTION_THRESHOLD, SpatialQueryHit, SyncEntry, SyncQueue,
    WIREFRAME_SEGMENTS, WORLD_MAX_BODIES, WireframeSeg, WorldEvent, debug_wireframe,
    debug_wireframe_world, wireframe_box, wireframe_capsule, wireframe_sphere,
};

#[cfg(all(feature = "physics", feature = "helpers"))]
pub use scenix_helpers::ColliderDebugHelper;

// v1.7.0 particle surface: CPU systems + the material/texture types the
// GPU draw path consumes. The dot-names collide with `scenix_material`'s
// `particle` module and `scenix_texture`'s `flipbook` module, so this is a
// curated list rather than a glob. Renderer particle APIs land through the
// `renderer` glob when both features are on.
#[cfg(feature = "particles")]
pub use scenix_particles::{
    ColorCurve, ColorKey, ColorOverLifetime, ConfineBox, DeterministicRng, Drag, EmissionShape,
    FlipbookConfig, Gravity, PARTICLE_POOL_LIMIT, Particle, ParticleEmitter, ParticleModifier,
    ParticleRenderBatch, ParticleRenderItem, ParticleSystem, ScalarCurve, ScalarKey,
    SizeOverLifetime, SpinOverLife,
};

// Node-surface re-exports are curated: scenix-nodes declares `graph`,
// `node`, and `value` modules plus its own `VertexAttribute` enum, all of
// which collide with the scene/mesh glob re-exports above.
#[cfg(feature = "nodes")]
pub use scenix_nodes::{
    ComponentIndex, FogStyle, GlslOptions, LightingStyle, NODE_GRAPH_LIMIT, NodeDiagnostic,
    NodeDiagnosticKind, NodeGlslProgram, NodeInput, NodeMaterial, NodeValue, NodeValueKind,
    NodeWgslProgram, PostNodeEffect, ShaderGraph, ShaderNodeData, ShaderNodeId, ShaderNodeKind,
    Stage, StageKind, ToneMapKind, UniformInput, WgslOptions, generate_material_glsl,
    generate_material_wgsl, generate_post_glsl, generate_post_wgsl,
};

#[cfg(feature = "nodes")]
pub use scenix_nodes::slots;

#[cfg(feature = "light")]
pub use scenix_light::*;

#[cfg(feature = "texture")]
pub use scenix_texture::*;

#[cfg(feature = "raycaster")]
pub use scenix_raycaster::*;

#[cfg(feature = "helpers")]
pub use scenix_helpers::*;

#[cfg(feature = "animato")]
pub use scenix_animato::*;

#[cfg(feature = "wasm")]
pub use scenix_wasm::*;

#[cfg(feature = "loader")]
pub use scenix_loader::*;

#[cfg(feature = "post")]
pub use scenix_post::*;

#[cfg(feature = "renderer")]
pub use scenix_renderer::*;

#[cfg(all(feature = "loader", feature = "renderer"))]
/// Counts returned after uploading an asset package into a renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UploadedAssetStats {
    /// Meshes registered.
    pub meshes: usize,
    /// Materials registered.
    pub materials: usize,
    /// Textures registered.
    pub textures: usize,
    /// Lights registered.
    pub lights: usize,
}

#[cfg(all(feature = "loader", feature = "renderer"))]
/// Convenience upload bridge from CPU asset packages to renderer-owned resources.
pub trait RendererAssetExt {
    /// Registers meshes, materials, textures, and lights from an asset package.
    fn register_asset_package(
        &mut self,
        package: &scenix_loader::AssetPackage,
    ) -> Result<UploadedAssetStats, scenix_core::ScenixError>;
}

#[cfg(all(feature = "loader", feature = "renderer"))]
impl RendererAssetExt for scenix_renderer::Renderer {
    fn register_asset_package(
        &mut self,
        package: &scenix_loader::AssetPackage,
    ) -> Result<UploadedAssetStats, scenix_core::ScenixError> {
        let mut stats = UploadedAssetStats::default();

        for (texture_id, texture) in &package.textures {
            let sampler = package
                .samplers
                .get(texture_id)
                .copied()
                .unwrap_or_default();
            self.register_texture2d(*texture_id, texture, sampler)?;
            stats.textures += 1;
        }
        for (texture_id, texture) in &package.texture_cubes {
            let sampler = package
                .samplers
                .get(texture_id)
                .copied()
                .unwrap_or_default();
            self.register_texture_cube(*texture_id, texture, sampler)?;
            stats.textures += 1;
        }

        if package.loaded_materials.is_empty() {
            for (material_id, material) in &package.materials {
                self.register_pbr_material(*material_id, material)?;
                stats.materials += 1;
            }
        } else {
            for (material_id, material) in &package.loaded_materials {
                match material {
                    scenix_loader::LoadedMaterial::Pbr(material) => {
                        self.register_pbr_material(*material_id, material)?;
                    }
                    scenix_loader::LoadedMaterial::Physical(material) => {
                        self.register_physical_material(*material_id, material)?;
                    }
                    scenix_loader::LoadedMaterial::Unlit(material) => {
                        self.register_unlit_material(*material_id, material)?;
                    }
                }
                stats.materials += 1;
            }
        }

        for (mesh_id, geometry) in &package.meshes {
            self.register_mesh(*mesh_id, geometry)?;
            stats.meshes += 1;
        }

        for (light_id, light) in &package.lights {
            match light {
                scenix_loader::LoadedLight::Directional(light) => {
                    self.register_directional_light(*light_id, *light)?;
                }
                scenix_loader::LoadedLight::Point(light) => {
                    self.register_point_light(*light_id, *light)?;
                }
                scenix_loader::LoadedLight::Spot(light) => {
                    self.register_spot_light(*light_id, *light)?;
                }
            }
            stats.lights += 1;
        }

        Ok(stats)
    }
}

/// Converts a loader-imported [`scenix_loader::LoadedAnimationClip`] into a
/// runtime [`scenix_animato::AnimationClip`], mapping glTF node indices to
/// scene [`scenix_core::NodeId`]s through `node_index_to_id`.
///
/// Requires both `loader` and `animato` features. The clip's channels use
/// typed [`scenix_animato::PropertyBinding`]s; unmapped node indices are
/// skipped. Cubic-spline channels fall back to linear sampling for v1.4.
#[cfg(all(feature = "loader", feature = "animato"))]
pub fn clip_from_loaded(
    loaded: &scenix_loader::LoadedAnimationClip,
    node_index_to_id: &[scenix_core::NodeId],
) -> scenix_animato::AnimationClip {
    use alloc::vec::Vec;
    use scenix_animato::{
        AnimationClip, ClipChannel, ClipTrack, KeyframeInterpolation, KeyframeQuat, KeyframeScalar,
        KeyframeVec3, NodeProperty, PropertyBinding,
    };
    use scenix_loader::{LoadedAnimationInterpolation, LoadedAnimationProperty};

    let interp = loaded
        .channels
        .first()
        .map(|c| match c.interpolation {
            LoadedAnimationInterpolation::Linear => KeyframeInterpolation::Linear,
            LoadedAnimationInterpolation::Step => KeyframeInterpolation::Step,
            LoadedAnimationInterpolation::CubicSpline => KeyframeInterpolation::CubicSpline,
        })
        .unwrap_or(KeyframeInterpolation::Linear);

    let mut channels = Vec::new();
    for ch in &loaded.channels {
        let Some(&node_id) = node_index_to_id.get(ch.node_index) else {
            continue;
        };

        // Decode packed `output` bytes into keyframe values. The loader stores
        // `output_components` per keyframe (1 scalar, 3 vec3, 4 quat/color).
        let key_count = ch.times.len();
        let comps = ch.output_components.max(1);

        let track = match ch.property {
            LoadedAnimationProperty::Translation | LoadedAnimationProperty::Scale => {
                let mut values = Vec::with_capacity(key_count);
                for k in 0..key_count {
                    let base = k * comps;
                    let x = ch.output.get(base).copied().unwrap_or(0.0);
                    let y = ch.output.get(base + 1).copied().unwrap_or(0.0);
                    let z = ch.output.get(base + 2).copied().unwrap_or(0.0);
                    values.push(scenix_math::Vec3::new(x, y, z));
                }
                ClipTrack::Vec3(KeyframeVec3::new(ch.times.clone(), values, interp))
            }
            LoadedAnimationProperty::Rotation => {
                let mut values = Vec::with_capacity(key_count);
                for k in 0..key_count {
                    let base = k * comps;
                    let x = ch.output.get(base).copied().unwrap_or(0.0);
                    let y = ch.output.get(base + 1).copied().unwrap_or(0.0);
                    let z = ch.output.get(base + 2).copied().unwrap_or(0.0);
                    let w = ch.output.get(base + 3).copied().unwrap_or(1.0);
                    values.push(scenix_math::Quat::new(x, y, z, w));
                }
                ClipTrack::Quat(KeyframeQuat::new(ch.times.clone(), values, interp))
            }
            LoadedAnimationProperty::MorphTargetWeights => {
                let mut values = Vec::with_capacity(key_count);
                for k in 0..key_count {
                    // First morph target weight per keyframe (multi-target
                    // morph clips are beyond v1.4 scope; first weight used).
                    values.push(ch.output.get(k * comps).copied().unwrap_or(0.0));
                }
                ClipTrack::Scalar(KeyframeScalar::new(ch.times.clone(), values, interp))
            }
        };

        let property = match ch.property {
            LoadedAnimationProperty::Translation => NodeProperty::Translation,
            LoadedAnimationProperty::Rotation => NodeProperty::Rotation,
            LoadedAnimationProperty::Scale => NodeProperty::Scale,
            LoadedAnimationProperty::MorphTargetWeights => NodeProperty::Scale,
        };

        channels.push(ClipChannel {
            binding: PropertyBinding::Node { node_id, property },
            track,
        });
    }

    let mut clip = AnimationClip {
        name: loaded.name.clone(),
        duration: loaded.duration,
        channels,
        markers: Vec::new(),
    };
    clip.recompute_duration();
    clip
}
