# `scenix-texture`

## Role

Raw CPU textures, samplers, mipmaps, atlases, and video-frame updates.

## Dependency Weight

Lightweight `no_std`; default facade feature.

## Install

```toml
[dependencies]
scenix-texture = "1"
```

## Key Public API

Texture2D, TextureCube, Texture3D, TextureFormat, Sampler, TextureAtlas, VideoTexture

## Common Use

```rust
use scenix_texture::{Sampler, TextureFormat};
let format = TextureFormat::Rgba8UnormSrgb;
let sampler = Sampler::default();
# let _ = (format, sampler);
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Added In

v1.7.0 adds `FlipbookGrid`, the shared frame-index to UV math for flipbook sheets.

v1.8.0 adds `EquirectSampler` (bilinear equirectangular sampling in the sky-bake row order) and `height_to_normal_map` (central-difference height-to-normal conversion) under the `environment_maps` module.


## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
