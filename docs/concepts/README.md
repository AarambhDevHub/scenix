# Concepts

These pages explain how Scenix systems fit together before you copy guide code. Read them when deciding crate boundaries, feature flags, data ownership, or runtime architecture.

- [Architecture Overview](architecture-overview.md)
- [Scene Graph](scene-graph.md)
- [Transforms](transforms.md)
- [Cameras](cameras.md)
- [Meshes And Geometry](meshes-and-geometry.md)
- [Materials](materials.md)
- [Lights](lights.md)
- [Textures](textures.md)
- [Renderer](renderer.md)
- [Post-Processing](post-processing.md)
- [Raycasting](raycasting.md)
- [Helpers](helpers.md)
- [Animation With Animato](animation-with-animato.md)
- [WASM And Browser](wasm-and-browser.md)
- [Feature Flags](feature-flags.md)
- [no_std](no-std.md)
- [Error Handling](error-handling.md)

## Common Rule

CPU crates describe scene data. Optional loader, renderer, post, Animato, and WASM crates are layered on top and stay opt-in.
