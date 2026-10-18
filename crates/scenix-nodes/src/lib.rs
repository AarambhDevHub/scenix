#![cfg_attr(not(feature = "std"), no_std)]

//! Typed shader node graphs and node materials for scenix.
//!
//! `scenix-nodes` is the optional shader-graph layer introduced in v1.6.0
//! ("Shader Nodes And Node Materials"). A [`ShaderGraph`] is a validated
//! directed acyclic graph of typed [`ShaderNodeKind`]s. Graphs are lowered by
//! deterministic backends into WGSL (the primary target, matching the
//! renderer's bind-group contract) or a constrained GLSL ES 3.00 subset for
//! WebGL hosts. [`NodeMaterial`] wraps a graph plus live CPU state and
//! implements the stable v1 [`scenix_material::Material`] trait, so node
//! materials flow through the same pipeline-key, sorting, and culling paths
//! as built-in materials. [`PostNodeEffect`] lowers full-screen graphs for
//! the post-processing stack.
//!
//! **Feature flags**
//!
//! | Feature | Default | Description |
//! | --- | --- | --- |
//! | `std` | yes | Standard-library support. |
//! | `serde` | no | Serialization for graphs, node materials, and post effects. |
//! | `inspector` | no | `Inspectable` snapshot support for the egui/WASM inspector model. |
//!
//! The crate compiles with `no_std`; the graph and both backends only
//! require `alloc`.
//!
//! # Example
//! ```
//! use scenix_material::Material as _;
//! use scenix_nodes::{LightingStyle, NodeMaterial, ShaderGraph, ShaderNodeKind, UniformInput};
//!
//! let mut graph = ShaderGraph::new();
//! let base_color = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
//! let shaded = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Toon(3)), base_color);
//! graph.add_fragment_output(shaded, None);
//!
//! let material = NodeMaterial::new("racing_toon", graph);
//! assert!(material.validate().is_empty());
//! let _ = material.pipeline_key();
//! ```
//!
//! Backends live in [`wgsl`] and [`glsl`]; renderer registration is exposed
//! by the `scenix` facade crate (feature `nodes`), and [`post`] chains into
//! `scenix-post`'s `NodeEffectProgram`.

extern crate alloc;

pub mod glsl;
pub mod graph;
#[cfg(feature = "inspector")]
mod inspector;
pub mod material;
pub mod node;
pub mod post;
pub mod value;
pub mod wgsl;

pub use glsl::{GlslOptions, NodeGlslProgram, generate_material_glsl, generate_post_glsl};
pub use graph::{NODE_GRAPH_LIMIT, NodeDiagnostic, NodeDiagnosticKind, ShaderGraph, ShaderNodeId};
pub use material::NodeMaterial;
pub use node::{
    ComponentIndex, FogStyle, LightingStyle, NodeInput, ShaderNodeData, ShaderNodeKind, Stage,
    StageKind, ToneMapKind, UniformInput, VertexAttribute,
};
pub use post::{PostNodeEffect, slots};
pub use value::{NodeValue, NodeValueKind};
pub use wgsl::{NodeWgslProgram, WgslOptions, generate_material_wgsl, generate_post_wgsl};
