# API Stability

Scenix v1.0.0 treats the public API as stable. Future releases should prefer additive changes, deprecate duplicate or experimental APIs before removal, and document migration steps.

## Stable Surface

The stable surface includes the facade feature names, focused crate boundaries, CPU authoring model, explicit renderer resource registration, loader output ownership, raycaster behavior, helper line geometry, Animato bridge role, and WASM wrapper scope.

## Deprecation Policy

Deprecated items should include `since = "1.0.0"` or the release that introduced the deprecation and a direct replacement note.

## User Rule

If you depend on optional systems, keep their features explicit in Cargo.toml so future audits are straightforward.
