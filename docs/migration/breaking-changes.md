# Breaking Changes

The v1.0.0 line is the stable baseline. Compared with 0.9.0, the main required change is dependency version selection.

## Expected Changes From 0.9

- Update Scenix crate versions to `1` or `1.0.0`.
- Keep optional feature choices explicit.
- Review renderer material preview limitations if your app relied on advanced physical shading claims.

## Not Changed

- The facade remains modular.
- Heavy GPU, loader, post, Animato, and WASM systems remain optional.
- Existing CPU authoring crates remain usable independently.
