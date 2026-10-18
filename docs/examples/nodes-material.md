# Node Material

## Purpose

Builds a typed shader graph into a `NodeMaterial`, checks its CPU state and
pipeline key, and generates text-free validation (no GPU required).

## Source

`examples/nodes_material.rs`

## Relevant Feature Flags

`nodes` (plus the default `material` feature for pipeline keys)

## Run Or Check

```sh
cargo run -p scenix --example nodes_material --features nodes
```

## What To Look For

- The lit graph validates with an empty diagnostic list.
- The node material pipeline key reports a `Custom` shader kind with
  `FEATURE_NODE_GRAPH` set, and a textured variant turns on
  `FEATURE_ALBEDO_TEXTURE`.
- Recoloring keeps the pipeline key, so live CPU state never rebuilds
  pipelines.

## Related Docs

- [Examples index](README.md)
- [scenix-nodes crate reference](../api/scenix-nodes.md)
- [Feature flags](../concepts/feature-flags.md)
