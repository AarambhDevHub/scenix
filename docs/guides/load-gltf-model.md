# Load A glTF Model

## Goal

Decode a glTF or GLB asset into scene, mesh, material, texture, light, and camera stores.

## Relevant Feature Flags

`loader`; add `renderer` when rendering the result.

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```rust
use scenix::GltfLoader;

let asset = GltfLoader::new().load_file("scene.gltf")?;
println!("meshes: {}", asset.meshes.len());
# Ok::<(), scenix::ScenixError>(())
```

## Verify

Run `cargo run -p scenix --example gltf_scene --features "loader renderer"`.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
