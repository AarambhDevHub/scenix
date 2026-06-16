# Performance

Use these pages when the app is large enough that compile time, asset loading, scene traversal, raycasting, rendering, or browser payload size matters.

For v1.3 asset-heavy apps, prefer `AssetManager` so repeated loads share cached `AssetPackage` handles, set a memory budget when importing many large files, and inspect package diagnostics before uploading resources to the renderer.

- [Compile Time](compile-time.md)
- [Crate Size](crate-size.md)
- [BVH Raycasting](bvh-raycasting.md)
- [Scene Graph Optimization](scene-graph-optimization.md)
- [Renderer Performance](renderer-performance.md)
- [WASM Performance](wasm-performance.md)
- [Benchmarking](benchmarking.md)
