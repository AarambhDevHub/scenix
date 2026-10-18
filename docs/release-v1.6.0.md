# Scenix v1.6.0 — Shader Nodes And Node Materials

Version 1.6 adds a graph layer above raw `ShaderMaterial` without weakening the
low-level WGSL escape hatch. Applications describe shading as typed,
serializable node graphs, validate them on the CPU, and let the same graph
lower to renderer-contract WGSL or WebGL-tooling GLSL. Renderer draws, post
effects, and browser preflight all speak the same generated modules.

## Data Flow

```text
 ShaderGraph  (nodes, ports, typed values)
        |
        v
 NodeMaterial (graph + CPU state, serdeable)
        |
        +---> generate_material_wgsl ---> Renderer::register_node_material
        |         |                              (lazy pipeline per slot)
        |         '--> scenix_wasm preflight ---> GLSL + diagnostics to JS
        |
        +---> generate_material_glsl ---> WebGL tooling (flattened uniforms)

 PostNodeEffect + slots  ---> generate_post_wgsl ---> scenix-post::NodeEffectProgram
                                                             (per-format pipeline,
                                                              [f32; 8] params)
```

Graphs validate over the live subgraph only. Dead nodes never reach codegen,
duplicated expressions collapse deterministically, and stage-incompatible
nodes surface specific `NodeDiagnostic`s instead of broken shaders.

## New Public Surfaces

- `scenix-nodes` (new crate): `ShaderGraph`, `ShaderNodeKind`, `NodeInput`,
  `NodeValue`/`NodeValueKind`, `Stage`, `NodeMaterial`, `PostNodeEffect`, the
  `slots` parameter map, stage gating, and the `NodeDiagnostic` tree.
- WGSL backend: `generate_material_wgsl` and `generate_post_wgsl` matching the
  renderer bind contract (groups 0-3, six varyings, alpha-cutoff discard, and
  displacement through the vertex output's local position).
- GLSL backend: `generate_material_glsl` and `generate_post_glsl` (GLSL ES
  3.00, flattened plain uniforms for the WebGL surface).
- `scenix-renderer` (behind `nodes`): `NodeMaterialSlot` registry,
  `register_node_material` / `unregister_node_material` on `GpuScene`, lazy
  node pipelines, per-draw `set_pipeline` switching, and culling fallback for
  unregistered node materials.
- `scenix-post` (behind `nodes`): `NodeEffectProgram::from_effect` /
  `with_source` and `run` with the `[f32; 8]` post parameter block.
- `scenix-wasm` (behind `nodes`): `compile_node_material_glsl` and
  `validate_node_material_json` preflight objects.
- `scenix` facade: the `nodes` feature forwards to `renderer`, `post`, and
  `wasm`, and re-exports the node API by name (curated list).

## Choosing A Feature

Enable `nodes` alone for authoring and codegen. Add `renderer` when node
materials must shade draws; add `post` for executable post-node effects; add
`wasm` for editor-grade preflight in the browser.

```toml
[dependencies]
scenix = { version = "1.6", features = ["nodes", "renderer", "post"] }
```

## Performance Design

- Graphs, validation, and codegen are CPU-only; nothing GPU-related is touched
  until a renderer or post program actually consumes generated WGSL.
- Node pipelines are built lazily once per `NodeMaterialSlot` shader hash and
  reused across frames and draws.
- Node uniforms pack into the standard material block at the prepared draw
  stride, so per-draw node switching adds no extra allocation.
- `NodeMaterial::pipeline_key` ignores live CPU state, so recoloring never
  invalidates pipelines.
- Generated modules keep the exact varyings of the built-in vertex layout, so
  a mixed builtin/node draw list shares one vertex buffer pipeline shape.
- `benches/nodes_bench.rs` covers chain and fan-out graph build/lower paths.

## Migration

No v1.5 interaction API changed. Built-in material WGSL is byte-identical to
v1.5.0, and the renderer frame uniform only gained the elapsed-seconds slot
(`frame.resolution` is now `[width, height, elapsed, 1 / height]`) — built-in
shaders never read it, and node shaders read `Time` from `frame.resolution.z`.
Applications adopt nodes incrementally: author a graph, generate WGSL, then
register a `NodeMaterial` id alongside built-in materials.

## Validation

The release is covered by the `scenix-nodes` unit and integration suites
(stage gating, bind contract, GLSL flattening, CSE dedup, displacement,
color-like rules), the facade `scenix_v16` suite, GPU-gated headless node
rendering plus mixed node/builtin draws (`nodes_gpu`, lavapipe/Vulkan in CI),
browser preflight tests, all facade feature lanes, no-default CPU builds,
`wasm32-unknown-unknown`, rustdoc warnings-as-errors, packaging, and
benchmarks.

Related reading:

- [scenix-nodes crate reference](api/scenix-nodes.md)
- [Examples](examples/README.md)
- [GitHub release notes](../.github/release-notes/1.6.0.md)
