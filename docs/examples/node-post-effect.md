# Node Post Effect

## Purpose

Compiles every bundled `PostNodeEffect` preset to WGSL and prints the stable
parameter-slot contract shared by `scenix-post::NodeEffectProgram`.

## Source

`examples/node_post_effect.rs`

## Relevant Feature Flags

`nodes`

## Run Or Check

```sh
cargo run -p scenix --example node_post_effect --features nodes
```

## What To Look For

- Every preset validates and compiles against the post pass contract:
  group 0 binding 0 source texture, binding 1 linear sampler, binding 2
  `PostParams` (`[f32; 8]`).
- The printed slot map shows where host-driven sliders (`vignette strength`,
  grade saturation/contrast) land inside the parameter block.
- Byte counts per preset illustrate the size of each generated module.

## Related Docs

- [Examples index](README.md)
- [scenix-nodes crate reference](../api/scenix-nodes.md)
- [Feature flags](../concepts/feature-flags.md)
