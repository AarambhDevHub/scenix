use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use scenix_camera::PerspectiveCamera;
use scenix_core::{MaterialId, MeshId, NodeId, ValidationError};
use scenix_math::{Mat3, Mat4, Ray3, Vec2};
use scenix_mesh::Geometry;
use scenix_scene::{NodeKind, SceneGraph};

use crate::{Bvh, BvhEntry, Intersection};

/// Provides mesh geometry by `MeshId`.
pub trait GeometryProvider {
    /// Returns geometry for `mesh_id`, if present.
    fn geometry(&self, mesh_id: MeshId) -> Option<&Geometry>;
}

impl GeometryProvider for BTreeMap<MeshId, Geometry> {
    #[inline]
    fn geometry(&self, mesh_id: MeshId) -> Option<&Geometry> {
        self.get(&mesh_id)
    }
}

impl GeometryProvider for [(MeshId, Geometry)] {
    fn geometry(&self, mesh_id: MeshId) -> Option<&Geometry> {
        self.iter()
            .find_map(|(id, geometry)| (*id == mesh_id).then_some(geometry))
    }
}

impl<const N: usize> GeometryProvider for [(MeshId, Geometry); N] {
    #[inline]
    fn geometry(&self, mesh_id: MeshId) -> Option<&Geometry> {
        self.as_slice().geometry(mesh_id)
    }
}

/// BVH-accelerated scene raycaster.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Raycaster {
    bvh: Option<Bvh>,
    layers: u32,
}

impl Raycaster {
    /// Creates a raycaster that tests all layers.
    #[inline]
    pub const fn new() -> Self {
        Self {
            bvh: None,
            layers: u32::MAX,
        }
    }

    /// Creates a raycaster with a layer mask.
    #[inline]
    pub const fn with_layers(layers: u32) -> Self {
        Self { bvh: None, layers }
    }

    /// Returns the active layer mask.
    #[inline]
    pub const fn layers(&self) -> u32 {
        self.layers
    }

    /// Sets the active layer mask. Existing BVH data remains valid but may
    /// contain now-filtered entries.
    #[inline]
    pub fn set_layers(&mut self, layers: u32) {
        self.layers = layers;
    }

    /// Returns the built BVH, if any.
    #[inline]
    pub const fn bvh(&self) -> Option<&Bvh> {
        self.bvh.as_ref()
    }

    /// Clears cached BVH data.
    #[inline]
    pub fn clear_bvh(&mut self) {
        self.bvh = None;
    }

    /// Builds a node-level BVH from visible mesh nodes in `scene`.
    pub fn build_bvh<G: GeometryProvider + ?Sized>(
        &mut self,
        scene: &SceneGraph,
        geometries: &G,
    ) -> Result<(), ValidationError> {
        let mut entries = Vec::new();
        for node_id in scene.iter_depth_first() {
            let Some((mesh_id, _material_id)) = mesh_node(scene, node_id, self.layers) else {
                continue;
            };
            let geometry = geometries
                .geometry(mesh_id)
                .ok_or(ValidationError::InvalidId)?;
            if geometry.is_empty() {
                continue;
            }
            let world = scene.world_matrix(node_id).unwrap_or(Mat4::IDENTITY);
            entries.push(BvhEntry::new(node_id, geometry.aabb().transform(world)));
        }
        self.bvh = Some(Bvh::build(&entries));
        Ok(())
    }

    /// Returns the nearest intersection, if any.
    pub fn cast_ray<G: GeometryProvider + ?Sized>(
        &self,
        ray: Ray3,
        scene: &SceneGraph,
        geometries: &G,
    ) -> Option<Intersection> {
        self.cast_ray_all(ray, scene, geometries).into_iter().next()
    }

    /// Returns all intersections sorted by ascending ray distance.
    pub fn cast_ray_all<G: GeometryProvider + ?Sized>(
        &self,
        ray: Ray3,
        scene: &SceneGraph,
        geometries: &G,
    ) -> Vec<Intersection> {
        let candidates = if let Some(bvh) = &self.bvh {
            bvh.traverse(ray)
        } else {
            mesh_nodes(scene, self.layers)
        };
        self.cast_candidates(ray, scene, geometries, &candidates)
    }

    /// Brute-force all-hit path used to validate BVH results.
    pub fn cast_ray_all_bruteforce<G: GeometryProvider + ?Sized>(
        &self,
        ray: Ray3,
        scene: &SceneGraph,
        geometries: &G,
    ) -> Vec<Intersection> {
        let candidates = mesh_nodes(scene, self.layers);
        self.cast_candidates(ray, scene, geometries, &candidates)
    }

    /// Builds a ray from perspective-camera normalized device coordinates.
    #[inline]
    pub fn from_camera_ndc(camera: &PerspectiveCamera, ndc: Vec2) -> Ray3 {
        camera.screen_to_ray(ndc)
    }

    fn cast_candidates<G: GeometryProvider + ?Sized>(
        &self,
        ray: Ray3,
        scene: &SceneGraph,
        geometries: &G,
        candidates: &[NodeId],
    ) -> Vec<Intersection> {
        let mut hits = Vec::new();
        for node_id in candidates {
            let Some((mesh_id, material_id)) = mesh_node(scene, *node_id, self.layers) else {
                continue;
            };
            let Some(geometry) = geometries.geometry(mesh_id) else {
                continue;
            };
            if geometry.is_empty() {
                continue;
            }
            let world = scene.world_matrix(*node_id).unwrap_or(Mat4::IDENTITY);
            let world_aabb = geometry.aabb().transform(world);
            if ray.intersect_aabb(world_aabb).is_none() {
                continue;
            }
            intersect_geometry(
                ray,
                *node_id,
                mesh_id,
                material_id,
                world,
                geometry,
                &mut hits,
            );
        }
        hits.sort_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then_with(|| a.node_id.get().cmp(&b.node_id.get()))
        });
        hits
    }
}

impl Default for Raycaster {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

fn mesh_node(scene: &SceneGraph, node_id: NodeId, layers: u32) -> Option<(MeshId, MaterialId)> {
    let node = scene.get(node_id)?;
    if !node.visible || node.layer & layers == 0 {
        return None;
    }
    match node.kind {
        NodeKind::Mesh {
            mesh_id,
            material_id,
        } => Some((mesh_id, material_id)),
        _ => None,
    }
}

fn mesh_nodes(scene: &SceneGraph, layers: u32) -> Vec<NodeId> {
    scene
        .iter_depth_first()
        .filter(|id| mesh_node(scene, *id, layers).is_some())
        .collect()
}

fn intersect_geometry(
    ray: Ray3,
    node_id: NodeId,
    mesh_id: MeshId,
    material_id: MaterialId,
    world: Mat4,
    geometry: &Geometry,
    hits: &mut Vec<Intersection>,
) {
    let normal_matrix = Mat3::from_mat4(world)
        .inverse()
        .map(Mat3::transpose)
        .unwrap_or_else(|| Mat3::from_mat4(world));

    if geometry.indices.is_empty() {
        for triangle in (0..geometry.positions.len()).step_by(3) {
            if triangle + 2 >= geometry.positions.len() {
                break;
            }
            intersect_triangle(
                ray,
                node_id,
                mesh_id,
                material_id,
                world,
                normal_matrix,
                geometry,
                triangle,
                triangle + 1,
                triangle + 2,
                hits,
            );
        }
    } else {
        for triangle in geometry.indices.chunks_exact(3) {
            let a = triangle[0] as usize;
            let b = triangle[1] as usize;
            let c = triangle[2] as usize;
            if a >= geometry.positions.len()
                || b >= geometry.positions.len()
                || c >= geometry.positions.len()
            {
                continue;
            }
            intersect_triangle(
                ray,
                node_id,
                mesh_id,
                material_id,
                world,
                normal_matrix,
                geometry,
                a,
                b,
                c,
                hits,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn intersect_triangle(
    ray: Ray3,
    node_id: NodeId,
    mesh_id: MeshId,
    material_id: MaterialId,
    world: Mat4,
    normal_matrix: Mat3,
    geometry: &Geometry,
    a: usize,
    b: usize,
    c: usize,
    hits: &mut Vec<Intersection>,
) {
    let wa = world.mul_vec3(geometry.positions[a]);
    let wb = world.mul_vec3(geometry.positions[b]);
    let wc = world.mul_vec3(geometry.positions[c]);
    let Some((distance, bary_uv)) = ray.intersect_triangle(wa, wb, wc) else {
        return;
    };
    let u = bary_uv.x;
    let v = bary_uv.y;
    let w = 1.0 - u - v;
    let point = ray.at(distance);
    let normal = if geometry.normals.len() == geometry.positions.len() {
        normal_matrix
            .mul_vec3(geometry.normals[a] * w + geometry.normals[b] * u + geometry.normals[c] * v)
            .normalize()
    } else {
        (wb - wa).cross(wc - wa).normalize()
    };
    let uv = if geometry.uvs.len() == geometry.positions.len() {
        geometry.uvs[a] * w + geometry.uvs[b] * u + geometry.uvs[c] * v
    } else {
        Vec2::ZERO
    };

    hits.push(Intersection {
        node_id,
        mesh_id,
        material_id,
        distance,
        point,
        normal,
        uv,
    });
}
