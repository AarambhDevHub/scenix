use std::fs;

use scenix::{AssetManager, ScenixError};

fn main() -> Result<(), ScenixError> {
    let path = std::env::temp_dir().join("scenix-example-asset-manager.gltf");
    fs::write(&path, empty_gltf()).expect("asset fixture");

    let mut manager = AssetManager::new();
    manager.set_memory_budget_bytes(Some(1024 * 1024));
    let package = manager.load_file(&path)?;

    println!(
        "cached={}, memory={} bytes, stale_invalidated={}",
        manager.len(),
        manager.memory_bytes(),
        manager.invalidate_stale()
    );
    println!(
        "package={} diagnostics={}",
        package.label,
        package.diagnostics.len()
    );
    Ok(())
}

fn empty_gltf() -> &'static str {
    r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"name":"root"}]}"#
}
