# Post Processing

## Purpose

Composes post effects over rendered color.

## Source

`examples/post_processing.rs`

## Relevant Feature Flags

renderer, post

## Run Or Check

```sh
cargo run -p scenix --example post_processing --features "renderer post"
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Feature flags](../concepts/feature-flags.md)
