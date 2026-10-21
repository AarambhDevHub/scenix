use scenix_camera::PerspectiveCamera;
use scenix_core::{MaterialId, ValidationError};
use scenix_math::Mat4;
use scenix_scene::{NodeKind, SceneGraph};

use crate::{DrawSubmission, GpuScene, NodeMaterialSlot};

/// Culling counters produced while collecting draw submissions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CullingStats {
    /// Mesh nodes encountered in the scene.
    pub scene_meshes: u32,
    /// Meshes accepted by culling.
    pub visible_meshes: u32,
    /// Meshes rejected by culling.
    pub culled_meshes: u32,
    /// Environment nodes (terrain/sky/water) encountered in the scene.
    pub scene_environments: u32,
    /// Environment nodes accepted by culling.
    pub visible_environments: u32,
}

/// Environment draw submission generated from a terrain/sky/water node.
///
/// Geometry lives in the environment slot itself, so culling carries only
/// the material id (the slot key), the node transform, and depth-sort data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentDrawSubmission {
    /// Environment material id (slot key).
    pub material_id: MaterialId,
    /// Node world transform.
    pub world_matrix: Mat4,
    /// Distance from camera position to bounds center.
    pub distance_to_camera: f32,
}

/// Collects visible mesh and environment draw submissions from a scene
/// graph.
///
/// Mesh draws and environment draws come back separately; the renderer
/// interleaves them (sky behind opaque meshes, terrain after opaque
/// meshes, water before the particle phase).
pub fn collect_visible_draws(
    scene: &SceneGraph,
    gpu_scene: &GpuScene,
    camera: &PerspectiveCamera,
) -> Result<
    (
        Vec<DrawSubmission>,
        Vec<EnvironmentDrawSubmission>,
        CullingStats,
    ),
    ValidationError,
> {
    let frustum = camera.frustum();
    let mut stats = CullingStats::default();
    let mut draws = Vec::new();
    #[cfg(feature = "environment")]
    let mut env_draws = Vec::new();
    #[cfg(not(feature = "environment"))]
    let env_draws = Vec::new();

    for node_id in scene.iter_depth_first() {
        let Some(node) = scene.get(node_id) else {
            continue;
        };
        if !node.visible {
            continue;
        }
        let world_matrix = scene
            .world_matrix(node_id)
            .unwrap_or_else(|| node.transform.to_mat4());
        match &node.kind {
            NodeKind::Mesh {
                mesh_id,
                material_id,
            } => {
                stats.scene_meshes += 1;
                let mesh = gpu_scene.mesh(*mesh_id).ok_or(ValidationError::InvalidId)?;
                let material = gpu_scene.material(*material_id);
                let node_material = gpu_scene.node_material(*material_id);
                if material.is_none() && node_material.is_none() {
                    return Err(ValidationError::InvalidId);
                }
                let transparent = material
                    .map(|material| material.is_transparent())
                    .unwrap_or_else(|| node_material.is_some_and(NodeMaterialSlot::is_transparent));
                let world_aabb = mesh.packed().aabb.transform(world_matrix);
                if !frustum.intersects_aabb(&world_aabb) {
                    stats.culled_meshes += 1;
                    continue;
                }

                stats.visible_meshes += 1;
                draws.push(DrawSubmission {
                    mesh_id: *mesh_id,
                    material_id: *material_id,
                    world_matrix,
                    world_aabb,
                    distance_to_camera: world_aabb.center().distance(camera.position),
                    transparent,
                    render_order: 0,
                });
            }
            NodeKind::Terrain { material_id, .. }
            | NodeKind::Sky { material_id, .. }
            | NodeKind::Water { material_id, .. } => {
                stats.scene_environments += 1;
                #[cfg(feature = "environment")]
                {
                    let slot = gpu_scene
                        .environment(*material_id)
                        .ok_or(ValidationError::InvalidId)?;
                    let world_aabb = slot.aabb().transform(world_matrix);
                    if !frustum.intersects_aabb(&world_aabb) {
                        continue;
                    }
                    stats.visible_environments += 1;
                    env_draws.push(EnvironmentDrawSubmission {
                        material_id: *material_id,
                        world_matrix,
                        distance_to_camera: world_aabb.center().distance(camera.position),
                    });
                }
                #[cfg(not(feature = "environment"))]
                {
                    let _ = (material_id, world_matrix);
                    // Environment draws need the `environment` feature;
                    // unsupported payloads skip silently.
                }
            }
            NodeKind::Empty
            | NodeKind::Group
            | NodeKind::Light { .. }
            | NodeKind::Camera { .. }
            | NodeKind::Sprite(_)
            // v1.9.0 CPU-bridge nodes ride along with the scene sync; the
            // renderer draws nothing for them (helpers/facade own the GFX).
            | NodeKind::XrAnchor { .. }
            | NodeKind::AudioSource { .. }
            | NodeKind::PhysicsBody { .. }
            | NodeKind::Particles { .. } => {}
        }
    }

    Ok((draws, env_draws, stats))
}
