# Deploy To GitHub Pages

## Goal

Build the static Leptos CSR website for `/scenix/`.

## Relevant Feature Flags

Website crate uses `scenix` with WASM support.

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```sh
cd website
trunk build --release --public-url /scenix/
```

## Verify

The Pages workflow uploads `website/dist` and deploys it as a static artifact.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
