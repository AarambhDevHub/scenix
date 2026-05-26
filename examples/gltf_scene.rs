use std::fs;
use std::path::PathBuf;

use scenix::{GltfLoader, PerspectiveCamera, Renderer, RendererConfig, ScenixError, Vec3};

fn main() -> Result<(), ScenixError> {
    pollster::block_on(run())
}

async fn run() -> Result<(), ScenixError> {
    let dir = generated_gltf_dir();
    let asset = GltfLoader::new().load_file(dir.join("triangle.gltf"))?;

    let mut renderer = Renderer::headless(RendererConfig::new(128, 128)).await?;
    for (mesh_id, geometry) in &asset.meshes {
        renderer.register_mesh(*mesh_id, geometry)?;
    }
    for (material_id, material) in &asset.materials {
        renderer.register_pbr_material(*material_id, material)?;
    }

    let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
        .position(Vec3::new(0.0, 0.0, 3.0))
        .target(Vec3::ZERO);
    let stats = renderer.render(&asset.scene, &camera)?;
    println!(
        "loaded {} meshes, rendered {} visible meshes",
        asset.meshes.len(),
        stats.visible_meshes
    );

    Ok(())
}

fn generated_gltf_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("scenix-example-gltf-scene");
    fs::create_dir_all(&dir).expect("example asset directory");
    fs::write(dir.join("triangle.bin"), triangle_bin()).expect("triangle bin");
    fs::write(dir.join("triangle.gltf"), triangle_gltf_json()).expect("triangle gltf");
    dir
}

fn triangle_gltf_json() -> String {
    String::from(
        r#"{
  "asset": {"version": "2.0"},
  "scene": 0,
  "scenes": [{"nodes": [0]}],
  "nodes": [{"name": "tri", "mesh": 0}],
  "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 1, "material": 0}]}],
  "materials": [{"name": "red", "pbrMetallicRoughness": {"baseColorFactor": [0.8, 0.2, 0.1, 1.0], "metallicFactor": 0.0, "roughnessFactor": 0.7}}],
  "buffers": [{"uri": "triangle.bin", "byteLength": 48}],
  "bufferViews": [
    {"buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962},
    {"buffer": 0, "byteOffset": 36, "byteLength": 12, "target": 34963}
  ],
  "accessors": [
    {"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0.0, 0.0, 0.0], "max": [1.0, 1.0, 0.0]},
    {"bufferView": 1, "componentType": 5125, "count": 3, "type": "SCALAR"}
  ]
}"#,
    )
}

fn triangle_bin() -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u32, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    bytes
}
