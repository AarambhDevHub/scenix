# Shadow Demo

## Purpose

Shows light and shadow resource setup.

## Source

`examples/shadow_demo.rs`

## Relevant Feature Flags

renderer

## Run Or Check

```sh
cargo run -p scenix --example shadow_demo --features renderer
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Feature flags](../concepts/feature-flags.md)
