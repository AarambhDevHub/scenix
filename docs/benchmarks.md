# Benchmarks

Benchmarks are intended to catch allocation and hot-path regressions. They are compile-gated in CI with:

```sh
cargo bench --workspace --no-run
```

Important benchmark areas:

- transform propagation and scene traversal
- primitive generation and geometry packing
- BVH build/traversal and exact triangle tests
- renderer pipeline cache and draw submission
- loader decode/cache paths
- post stack setup
- Animato driver tick for many nodes/materials

GPU-heavy post and renderer benches should remain opt-in through environment variables so normal CI stays reliable.
