use scenix_camera::PerspectiveCamera;
use scenix_core::ValidationError;
use scenix_scene::{NodeKind, SceneGraph};

use crate::{DrawSubmission, GpuScene};

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
}

/// Collects visible draw submissions from a scene graph.
pub fn collect_visible_draws(
    scene: &SceneGraph,
    gpu_scene: &GpuScene,
    camera: &PerspectiveCamera,
) -> Result<(Vec<DrawSubmission>, CullingStats), ValidationError> {
    let frustum = camera.frustum();
    let mut stats = CullingStats::default();
    let mut draws = Vec::new();

    for node_id in scene.iter_depth_first() {
        let Some(node) = scene.get(node_id) else {
            continue;
        };
        if !node.visible {
            continue;
        }
        let NodeKind::Mesh {
            mesh_id,
            material_id,
        } = node.kind
        else {
            continue;
        };

        stats.scene_meshes += 1;
        let mesh = gpu_scene.mesh(mesh_id).ok_or(ValidationError::InvalidId)?;
        let material = gpu_scene
            .material(material_id)
            .ok_or(ValidationError::InvalidId)?;
        let world_matrix = scene
            .world_matrix(node_id)
            .unwrap_or_else(|| node.transform.to_mat4());
        let world_aabb = mesh.packed().aabb.transform(world_matrix);
        if !frustum.intersects_aabb(&world_aabb) {
            stats.culled_meshes += 1;
            continue;
        }

        stats.visible_meshes += 1;
        draws.push(DrawSubmission {
            mesh_id,
            material_id,
            world_matrix,
            world_aabb,
            distance_to_camera: world_aabb.center().distance(camera.position),
            transparent: material.is_transparent(),
            render_order: 0,
        });
    }

    Ok((draws, stats))
}
