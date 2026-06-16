use std::fs;
use std::path::PathBuf;

use scenix::{AssetManager, RendererAssetExt, RendererConfig, ScenixError};

fn main() -> Result<(), ScenixError> {
    pollster::block_on(run())
}

async fn run() -> Result<(), ScenixError> {
    let path = generated_gltf();
    let mut manager = AssetManager::new();
    let package = manager.load_file(&path)?;

    let mut renderer = scenix::Renderer::headless(RendererConfig::new(128, 128)).await?;
    let uploaded = renderer.register_asset_package(&package)?;

    println!(
        "package={}, meshes={}, materials={}, uploaded_meshes={}",
        package.label,
        package.meshes.len(),
        package.materials.len(),
        uploaded.meshes
    );
    Ok(())
}

fn generated_gltf() -> PathBuf {
    let dir = std::env::temp_dir().join("scenix-example-asset-pipeline");
    fs::create_dir_all(&dir).expect("asset dir");
    fs::write(dir.join("mesh.bin"), triangle_bin()).expect("mesh bin");
    fs::write(
        dir.join("scene.gltf"),
        r#"{
  "asset": {"version": "2.0"},
  "scene": 0,
  "scenes": [{"nodes": [0]}],
  "nodes": [{"name": "tri", "mesh": 0}],
  "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 1, "material": 0}]}],
  "materials": [{"name": "white", "pbrMetallicRoughness": {"baseColorFactor": [1.0, 1.0, 1.0, 1.0]}}],
  "buffers": [{"uri": "mesh.bin", "byteLength": 48}],
  "bufferViews": [
    {"buffer": 0, "byteOffset": 0, "byteLength": 36},
    {"buffer": 0, "byteOffset": 36, "byteLength": 12}
  ],
  "accessors": [
    {"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0, 0, 0], "max": [1, 1, 0]},
    {"bufferView": 1, "componentType": 5125, "count": 3, "type": "SCALAR"}
  ]
}"#,
    )
    .expect("scene gltf");
    dir.join("scene.gltf")
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
