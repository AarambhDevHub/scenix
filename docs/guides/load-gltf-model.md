# Load A glTF Model

## Goal

Decode a glTF or GLB asset into scene, mesh, material, texture, light, camera, and v1.3 asset-pipeline metadata stores.

## Relevant Feature Flags

`loader`; add `renderer` when rendering the result.

## Steps

1. Add the required Cargo features.
2. Use `AssetManager` or `GltfLoader::load_package_file` when you need v1.3 metadata.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```rust
use scenix::AssetManager;

let mut manager = AssetManager::new();
let package = manager.load_file("scene.gltf")?;
println!("meshes: {}", package.meshes.len());
# Ok::<(), scenix::ScenixError>(())
```

Use `GltfLoader::load_file` when you only need the older `GltfAsset` shape.

## Verify

Run `cargo run -p scenix --example asset_pipeline --features "loader renderer"`.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
