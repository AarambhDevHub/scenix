# Installation

This page explains the supported ways to depend on Scenix v1.5.0. Use the facade
crate for application code and focused crates when you are building a library
or a very small tool.

## Facade Install

```toml
[dependencies]
scenix = "1"
```

The default facade is CPU-first. It does not pull loader, renderer, post-processing, Animato, or browser dependencies unless you enable those features.

## Optional Systems

```toml
[dependencies]
scenix = { version = "1", features = ["loader"] }
scenix = { version = "1", features = ["renderer"] }
scenix = { version = "1", features = ["renderer", "post"] }
scenix = { version = "1", features = ["animato"] }
scenix = { version = "1", features = ["wasm"] }
scenix = { version = "1", features = ["interaction"] }
scenix = { version = "1", features = ["editor", "egui"] }
```

## Selected Crates

```toml
[dependencies]
scenix-math = "1"
scenix-scene = "1"
scenix-camera = "1"
scenix-mesh = "1"
scenix-raycaster = "1"
```

Use selected crates for libraries that should not expose the full facade dependency surface.

## no_std CPU Setup

```toml
[dependencies]
scenix-math = { version = "1", default-features = false, features = ["libm"] }
scenix-core = { version = "1", default-features = false }
scenix-scene = { version = "1", default-features = false }
scenix-camera = { version = "1", default-features = false }
```

Loader, renderer, post, and WASM paths are `std`-oriented.

## Feature Matrix

| Feature | Default | Use it for |
| --- | --- | --- |
| `std` | yes | Standard-library support for CPU crates. |
| `scene`, `camera`, `mesh`, `material`, `light`, `texture` | yes | CPU scene authoring. |
| `raycaster`, `helpers` | yes | Picking and debug helper geometry. |
| `interaction` | no | Controls, selection, dragging, transforms, and gizmos. |
| `editor` | no | Typed inspector snapshots for editor-facing systems. |
| `egui` | no | Read-only egui adapter for inspector snapshots. |
| `loader` | no | Asset packages, asset manager, glTF extension metadata, importers, and exporters. |
| `renderer` | no | `wgpu` surface and headless rendering. |
| `post` | no | GPU post-processing stack; normally used with `renderer`. |
| `animato` | no | Animato 1.7 bridge and clip-based animation runtime. |
| `wasm` | no | Browser canvas wrapper, DOM input mapping, WebGPU path, WebGL2 full fallback, and WebGL1 reduced fallback. |
| `serde` | no | Serialization support where each crate supports it. |
