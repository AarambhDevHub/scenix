use std::path::{Path, PathBuf};

use scenix_core::{LoadError, ScenixError, TextureId};
use scenix_math::{Vec2, Vec3};
use scenix_mesh::Geometry;

/// OBJ material metadata relevant to scenix CPU assets.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjMaterial {
    /// Source material name.
    pub name: String,
    /// Optional diffuse/base-color texture path.
    pub diffuse_texture: Option<PathBuf>,
    /// Stable texture identifier assigned from material order.
    pub texture_id: Option<TextureId>,
}

/// OBJ file decoded into one geometry per source model.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjAsset {
    /// Model geometries in source order.
    pub geometries: Vec<Geometry>,
    /// Material metadata loaded from the MTL file, if any.
    pub materials: Vec<ObjMaterial>,
    /// Geometry material indices in source order.
    pub geometry_materials: Vec<Option<usize>>,
}

/// Loads OBJ geometry, triangulating polygon faces.
pub fn load(path: impl AsRef<Path>) -> Result<Vec<Geometry>, ScenixError> {
    Ok(load_with_materials(path)?.geometries)
}

/// Loads OBJ geometry plus MTL material/texture metadata.
pub fn load_with_materials(path: impl AsRef<Path>) -> Result<ObjAsset, ScenixError> {
    let path = path.as_ref();
    let options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };
    let (models, materials) = tobj::load_obj(path, &options).map_err(|err| match err {
        tobj::LoadError::OpenFileFailed => LoadError::NotFound,
        _ => LoadError::Parse,
    })?;

    let mut asset = ObjAsset::default();
    asset.geometries.reserve(models.len());
    asset.geometry_materials.reserve(models.len());
    for model in models {
        asset.geometry_materials.push(model.mesh.material_id);
        asset.geometries.push(geometry_from_obj_mesh(&model.mesh)?);
    }

    if let Ok(materials) = materials {
        asset.materials.reserve(materials.len());
        for (index, material) in materials.into_iter().enumerate() {
            let diffuse_texture = material.diffuse_texture.and_then(|texture| {
                if texture.is_empty() {
                    None
                } else {
                    Some(resolve_relative(path, &texture))
                }
            });
            asset.materials.push(ObjMaterial {
                name: material.name,
                diffuse_texture: diffuse_texture.clone(),
                texture_id: diffuse_texture.map(|_| TextureId::new(index as u64 + 1)),
            });
        }
    }

    Ok(asset)
}

fn geometry_from_obj_mesh(mesh: &tobj::Mesh) -> Result<Geometry, ScenixError> {
    let mut geometry = Geometry::new();
    geometry.positions.reserve(mesh.positions.len() / 3);
    geometry.normals.reserve(mesh.normals.len() / 3);
    geometry.uvs.reserve(mesh.texcoords.len() / 2);
    geometry.indices.extend(mesh.indices.iter().copied());

    for position in mesh.positions.chunks_exact(3) {
        geometry
            .positions
            .push(Vec3::new(position[0], position[1], position[2]));
    }
    for normal in mesh.normals.chunks_exact(3) {
        geometry
            .normals
            .push(Vec3::new(normal[0], normal[1], normal[2]).normalize());
    }
    for uv in mesh.texcoords.chunks_exact(2) {
        geometry.uvs.push(Vec2::new(uv[0], uv[1]));
    }

    if geometry.normals.is_empty() {
        geometry.compute_normals();
    }
    geometry.validate()?;
    Ok(geometry)
}

fn resolve_relative(source: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        source.parent().unwrap_or_else(|| Path::new("")).join(path)
    }
}
