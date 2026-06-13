# Migration From 0.9 To 1.0

Scenix 1.0.0 stabilizes the modular API introduced during the 0.x milestones. Most 0.9 code should continue to compile after version bumps, because v1 favors additive APIs and deprecations over silent removal.

## Cargo Changes

```toml
[dependencies]
scenix = "1"
```

Keep optional features explicit:

```toml
scenix = { version = "1", features = ["renderer", "post"] }
```

## Review Checklist

- Replace `0.9` dependency requirements with `1`.
- Confirm renderer, loader, post, Animato, and WASM features are enabled only where needed.
- Re-run no-default checks for CPU crates if you support `no_std`.
- Review current renderer behavior and limitations in `../release-v1.2.0.md`.

## Verification

```sh
cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```
