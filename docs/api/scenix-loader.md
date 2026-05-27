# `scenix-loader`

## Role

Optional CPU asset loaders for glTF/GLB, OBJ/MTL, STL, images, KTX2, HDR/EXR, and asset caching.

## Dependency Weight

Heavy `std` path; enable `loader` on facade. `http` gates URL loading.

## Install

```toml
[dependencies]
scenix-loader = "1"
```

## Key Public API

GltfLoader, GltfAsset, AssetCache, LoaderOptions, obj, stl, image, hdr, ktx2

## Common Use

```rust
use scenix_loader::GltfLoader;
let loader = GltfLoader::new();
# let _ = loader;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
