# `scenix-core`

## Role

Shared IDs, colors, errors, and traits used across crates.

## Dependency Weight

Lightweight `no_std`.

## Install

```toml
[dependencies]
scenix-core = "1"
```

## Key Public API

NodeId, MeshId, MaterialId, TextureId, LightId, CameraId,
XrSessionId, XrAnchorId, AudioSourceId, AudioListenerId, PhysicsWorldId,
PhysicsBodyId, ColliderId, Color, ScenixError, ValidationError

## Common Use

```rust
use scenix_core::{Color, MeshId};
let mesh = MeshId::new(1);
let color = Color::rgb(1.0, 0.8, 0.2);
# let _ = (mesh, color);
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Added In

v1.7.0 adds the `ParticleSystemId` newtype alongside the other resource ids.

v1.8.0 adds the `TerrainId`, `SkyId`, and `WaterId` newtypes next to the other resource ids.

v1.9.0 adds the runtime-bridge newtypes — `XrSessionId`, `XrAnchorId`,
`AudioSourceId`, `AudioListenerId`, `PhysicsWorldId`, `PhysicsBodyId`, and
`ColliderId` — so session, anchor, audio, and simulation payloads key scene
nodes and helper payloads through the same opaque-id discipline (ids start
at 1 for physics bodies; `.0` stays private, read through `.get()`).

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
