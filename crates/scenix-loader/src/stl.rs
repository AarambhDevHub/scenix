use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

use scenix_core::{LoadError, ScenixError};
use scenix_math::Vec3;
use scenix_mesh::Geometry;

/// Loads an ASCII or binary STL file into triangle geometry.
pub fn load(path: impl AsRef<Path>) -> Result<Geometry, ScenixError> {
    let file = File::open(path.as_ref()).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            LoadError::NotFound
        } else {
            LoadError::Io
        }
    })?;
    let mut reader = BufReader::new(file);
    load_reader(&mut reader)
}

/// Loads STL bytes into triangle geometry.
pub fn load_bytes(bytes: &[u8]) -> Result<Geometry, ScenixError> {
    let mut reader = Cursor::new(bytes);
    load_reader(&mut reader)
}

fn load_reader<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Result<Geometry, ScenixError> {
    let mesh = stl_io::read_stl(reader).map_err(|_| LoadError::Parse)?;
    let mut geometry = Geometry::new();
    geometry.positions.reserve(mesh.faces.len() * 3);
    geometry.normals.reserve(mesh.faces.len() * 3);
    geometry.indices.reserve(mesh.faces.len() * 3);

    for face in mesh.faces {
        let normal = Vec3::new(face.normal[0], face.normal[1], face.normal[2]).normalize();
        for vertex_index in face.vertices {
            let vertex = mesh
                .vertices
                .get(vertex_index)
                .ok_or(ScenixError::Load(LoadError::Parse))?;
            geometry
                .positions
                .push(Vec3::new(vertex[0], vertex[1], vertex[2]));
            geometry.normals.push(normal);
            geometry.indices.push(geometry.indices.len() as u32);
        }
    }

    if geometry.normals.iter().all(|normal| *normal == Vec3::ZERO) {
        geometry.compute_normals();
    }
    geometry.validate()?;
    Ok(geometry)
}
