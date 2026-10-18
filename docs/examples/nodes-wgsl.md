# Node WGSL Backend

## Purpose

Lower one graph through both backends: the renderer-contract WGSL module and
the GLSL ES 3.00 module, including a vertex displacement port.

## Source

`examples/nodes_wgsl.rs`

## Relevant Feature Flags

`nodes` (plus the default `material` feature for pipeline keys)

## Run Or Check

```sh
cargo run -p scenix --example nodes_wgsl --features nodes
```

## What To Look For

- The WGSL module binds groups 0..=3 per the renderer contract and declares
  `struct MaterialUniform`.
- Displacement text reaches both backends through the vertex output's local
  position field.
- The GLSL fragment stage is GLSL ES 3.00 with flattened uniforms.
- The full WGSL source is printed, so you can diff it against the built-in
  material contract.

## Related Docs

- [Examples index](README.md)
- [scenix-nodes crate reference](../api/scenix-nodes.md)
- [Feature flags](../concepts/feature-flags.md)
