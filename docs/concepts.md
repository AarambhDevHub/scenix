# Concepts

## Modular Crates

Scenix keeps each subsystem in a focused crate. CPU authoring crates are lightweight and remain usable without the GPU stack. Loader, renderer, post-processing, Animato, and WASM support are optional.

## Scene Data And GPU Data

`SceneGraph` stores scene hierarchy, transforms, mesh IDs, material IDs, layers, and visibility. It does not own GPU buffers. The renderer has explicit resource registration methods such as `register_mesh`, `register_pbr_material`, and `register_texture2d`.

## Transform Updates

After changing transforms or hierarchy, call `update_world_transforms()`. The v1 path deduplicates dirty roots and traverses children without cloning the child list on each subtree update.

## Picking

`scenix-raycaster` builds a node-level BVH over visible world-space mesh bounds, then exact-tests candidate triangles. Users supply geometry through a provider, commonly a `BTreeMap<MeshId, Geometry>`.

## Helpers

`scenix-helpers` generates `LineGeometry` for grids, axes, bounding boxes, cameras, lights, and skeletons. Renderer line topology is not required; helpers remain data generators.

## Stable API Policy

The v1.0.0 API prefers additive changes. Duplicate or experimental APIs should be deprecated before removal.
