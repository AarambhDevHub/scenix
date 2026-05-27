# Troubleshooting

## WebGPU Demo Does Not Start

Check browser WebGPU support. The website uses a fallback canvas when WebGPU is unavailable.

## Renderer Test Fails In CI

Run GPU tests only on a configured backend:

```sh
SCENIX_RUN_GPU_TESTS=1 WGPU_BACKEND=vulkan cargo test -p scenix-renderer -p scenix-post --all-features
```

## Loader Cannot Decode Asset

Confirm the loader feature and format support. `scenix-loader` decodes assets into CPU data; it does not upload textures to the renderer.

## Raycaster Misses Objects

Call `scene.update_world_transforms()` after transform edits and rebuild the BVH after scene or geometry changes.

## no_std Build Fails

Disable default features on CPU crates and do not include loader, renderer, post, or WASM crates in the no-default target.
