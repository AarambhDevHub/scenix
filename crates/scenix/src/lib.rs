#![cfg_attr(not(feature = "std"), no_std)]

//! Facade crate for scenix Renderer APIs.
//!
//! This release re-exports the Foundation crates, the GPU-free scene graph,
//! CPU-side geometry, materials, lights, textures, cameras, optional loaders,
//! raycasting, debug helper geometry, optional Animato and WASM integration,
//! optional post-processing, and optional renderer APIs.

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
