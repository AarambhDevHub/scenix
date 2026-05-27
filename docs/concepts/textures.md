# Textures

## Purpose

Store raw CPU pixel data, samplers, mipmaps, atlases, and video-frame updates.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Default facade features include `texture`; image decoding needs `loader`.

## Key Rules

- Texture crates do not decode PNG/JPEG by themselves.
- Use `scenix-loader` for image files.
- Register textures explicitly with the renderer.


## Example

```rust
use scenix::{Sampler, Texture2D, TextureFormat};

let data = vec![255; 4 * 4 * 4];
let texture = Texture2D::new(4, 4, TextureFormat::Rgba8UnormSrgb, data).unwrap();
let sampler = Sampler::default();
# let _ = (texture, sampler);
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
