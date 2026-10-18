# `scenix-nodes`

## Role

Optional typed shader graphs, node materials, and post node effects with WGSL
and GLSL ES 3.00 code generation. Added in scenix v1.6.0.

## Dependency Weight

Lightweight `no_std` crate (add the `std` and/or `serde` features as needed);
depends only on `scenix-math`, `scenix-core`, and `scenix-material`. It is not
a default facade crate; enable it with the `nodes` feature.

## Install

```toml
[dependencies]
scenix = { version = "1", features = ["nodes"] }
# or the focused crate:
scenix-nodes = "1"
```

## Key Public API

- Graphs: `ShaderGraph`, `ShaderNodeId`, `Stage`, `NODE_GRAPH_LIMIT`.
- Nodes: `ShaderNodeKind`, `NodeInput`, `NodeValue`, `NodeValueKind`,
  `UniformInput`, `VertexAttribute`, `ComponentIndex`.
- Materials: `NodeMaterial`, `LightingStyle`, `ToneMapKind`, `FogStyle`.
- Post: `PostNodeEffect` plus its atomic presets and the stable parameter
  slots under `slots`.
- Backends: `generate_material_wgsl` / `generate_post_wgsl` (`NodeWgslProgram`,
  `WgslOptions`) and `generate_material_glsl` / `generate_post_glsl`
  (`NodeGlslProgram`, `GlslOptions`).
- Diagnostics: `NodeDiagnostic`, `NodeDiagnosticKind` — every failure is a
  typed value, never a panic.

## Common Use

```rust
use scenix_nodes::{
    generate_material_wgsl, LightingStyle, Material, NodeInput, NodeMaterial, ShaderGraph,
    ShaderNodeKind, UniformInput, WgslOptions,
};

let mut graph = ShaderGraph::new();
let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
let time = graph.push(ShaderNodeKind::Uniform(UniformInput::Time));
let pulse = graph.add(
    ShaderNodeKind::Multiply,
    vec![NodeInput::Node(base), NodeInput::Node(time)],
);
let lit = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), pulse);
graph.add_fragment_output(lit, None);

let material = NodeMaterial::new("pulse", graph);
assert!(material.validate().is_empty());

let wgsl = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
assert!(wgsl.source().contains("@group(2)"));
let _ = Material; // NodeMaterial reports the shader-material pipeline key
```

With the facade, the same surface is available via `scenix::Material` —
`NodeMaterial`, `ShaderGraph`, `generate_material_wgsl`, and friends are
re-exported by name (see the curated facade re-exports in
[ARCHITECTURE.md](../../ARCHITECTURE.md)). The one name to qualify explicitly
is `scenix_nodes::VertexAttribute`, because `scenix` also re-exports the mesh
crate's `VertexAttribute` glob.

## Validation And Diagnostics

- `NodeMaterial::validate` covers the fragment stage.
- `ShaderGraph::validate(Stage::Vertex)` additionally covers displacement
  ports; `ShaderGraph::validate(Stage::Post)` gates post-only nodes.
- Graphs that fail produce `NodeDiagnostic { kind, node }` values such as
  `StageUnsupported`, port-shape mismatches, cycles, and `UnknownNode`.
- Registration into a renderer re-validates: broken graphs and null ids are
  rejected with `ScenixError` rather than compiled.

## Notes

Use this crate directly when you need its boundary in your own public API. Use
the `scenix` facade when building an application and you want one stable import
surface. Renderer, post stack, and browser preflight integration turn on
automatically when those optional features share a build with `nodes`.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
- [Materials guide](../materials-guide.md)
- [v1.6.0 release guide](../release-v1.6.0.md)
