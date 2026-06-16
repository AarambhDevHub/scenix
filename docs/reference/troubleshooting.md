# Troubleshooting

## Browser Demo Does Not Start

Check browser WebGPU and WebGL support. The website tries WebGPU first, WebGL2 second, reduced WebGL1 third, then uses a Canvas2D preview when both GPU paths are unavailable.

## Renderer Test Fails In CI

Run GPU tests only on a configured backend:

```sh
SCENIX_RUN_GPU_TESTS=1 WGPU_BACKEND=vulkan cargo test -p scenix-renderer -p scenix-post --all-features
```

## Loader Cannot Decode Asset

Confirm the loader feature and format support. In v1.3, `AssetPackage::diagnostics` reports recognized-but-unsupported features such as Draco or meshopt compression. `scenix-loader` decodes assets into CPU data; upload remains explicit through renderer registration or `RendererAssetExt`.

## Raycaster Misses Objects

Call `scene.update_world_transforms()` after transform edits and rebuild the BVH after scene or geometry changes.

## no_std Build Fails

Disable default features on CPU crates and do not include loader, renderer, post, or WASM crates in the no-default target.
