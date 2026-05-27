# Run Headless Render Tests

## Use When

You need this behavior inside an app and want the smallest Scenix subsystem set that supports it.

## Approach

Gate GPU tests with an environment variable so CI can choose lavapipe or skip GPU work.

## Example

```sh
SCENIX_RUN_GPU_TESTS=1 WGPU_BACKEND=vulkan cargo test -p scenix-renderer --all-features
```

## Verify

Add a focused test around the state change or command shown above. For browser or GPU paths, keep tests gated so normal CPU CI remains fast.
