# `scenix-loader`

## Role

Optional CPU asset packages, asset manager, importers, exporters, diagnostics, dependency tracking, glTF/GLB extension metadata, OBJ/MTL, STL, images, KTX2, HDR/EXR, and asset caching.

## Dependency Weight

Heavy `std` path; enable `loader` on facade. `http` gates URL loading.

## Install

```toml
[dependencies]
scenix-loader = "1"
```

## Key Public API

GltfLoader, GltfAsset, AssetPackage, AssetManager, AssetCache, LoaderOptions, AssetDiagnostic, LoadedAnimationClip, LoadedSkin, LoadedMaterial, TextureTransform, MaterialVariant, RendererAssetExt through the facade, obj, stl, image, hdr, ktx2, export

## Common Use

```rust
use scenix_loader::{AssetManager, export};

# fn run() -> Result<(), scenix_core::ScenixError> {
let mut manager = AssetManager::new();
let package = manager.load_file("scene.glb")?;
println!("{}", export::scene_json_string(&package));
# Ok(())
# }
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

`GltfAsset` remains source-compatible. Use `AssetPackage` when you need v1.3 sidecars for skins, morph targets, imported animation metadata, material extensions, dependency graphs, diagnostics, exporters, or explicit renderer upload through `RendererAssetExt`.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
