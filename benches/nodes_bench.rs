//! Micro-benchmarks for scenix-nodes validation and lowering.
//!
//! Runs standalone (`harness = false`):
//!
//! ```text
//! cargo bench -p scenix-nodes --bench nodes_bench
//! ```

use std::time::Instant;

use scenix_material::Material as _;
use scenix_nodes::glsl::{GlslOptions, generate_material_glsl};
use scenix_nodes::wgsl::{WgslOptions, generate_material_wgsl, generate_post_wgsl};
use scenix_nodes::{
    NodeInput, NodeMaterial, PostNodeEffect, ShaderGraph, ShaderNodeKind, Stage, UniformInput,
};

/// Builds a deep chain of saturate/one-minus nodes over `depth` levels.
fn build_chain(depth: usize) -> ShaderGraph {
    let mut graph = ShaderGraph::new();
    let mut current = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    for _ in 0..depth {
        current = graph.chain(ShaderNodeKind::Saturate, current);
        current = graph.chain(ShaderNodeKind::OneMinus, current);
    }
    graph.add_fragment_output(current, None);
    graph
}

/// Builds a wide fan: one shared base color feeding `fanout` chains that
/// multiply against a shared emissive uniform.
fn build_fan(fanout: usize, depth: usize) -> ShaderGraph {
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let emissive = graph.push(ShaderNodeKind::Uniform(UniformInput::Emissive));
    let mut tail = base;
    for _ in 0..fanout {
        let mut current = base;
        for _ in 0..depth {
            current = graph.add(
                ShaderNodeKind::Multiply,
                vec![NodeInput::Node(current), NodeInput::Node(emissive)],
            );
        }
        tail = current;
    }
    graph.add_fragment_output(tail, None);
    graph
}

fn main() {
    let iterations = 128u32;

    let deep: Vec<ShaderGraph> = (0..4).map(|i| build_chain(64 + i * 16)).collect();
    let fan: ShaderGraph = build_fan(8, 16);
    let materials: Vec<NodeMaterial> = deep
        .iter()
        .map(|graph| NodeMaterial::new("bench", graph.clone()))
        .collect();
    let post_effects: Vec<PostNodeEffect> = vec![
        PostNodeEffect::vignette(),
        PostNodeEffect::desaturate(),
        PostNodeEffect::color_grade(),
    ];

    let mut start = Instant::now();
    for _ in 0..iterations {
        for graph in &deep {
            let _ = graph.validate(Stage::Fragment);
        }
    }
    let elapsed = start.elapsed();
    println!(
        "validate deep x{}: {:?} ({:?}/call)",
        iterations,
        elapsed,
        elapsed / (iterations * deep.len() as u32)
    );

    start = Instant::now();
    for _ in 0..iterations {
        let _ = fan.validate(Stage::Fragment);
    }
    println!("validate fan x{}: {:?}", iterations, start.elapsed());

    start = Instant::now();
    for _ in 0..iterations {
        for graph in &deep {
            let _ = graph.evaluation_order();
        }
    }
    println!("evaluation_order x{}: {:?}", iterations, start.elapsed());

    start = Instant::now();
    for _ in 0..iterations {
        for graph in &deep {
            let _ = graph.shader_hash();
        }
    }
    println!("shader_hash x{}: {:?}", iterations, start.elapsed());

    start = Instant::now();
    for _ in 0..iterations {
        for material in &materials {
            let _ = generate_material_wgsl(material, &WgslOptions::default());
        }
    }
    println!("material wgsl x{}: {:?}", iterations, start.elapsed());

    start = Instant::now();
    for _ in 0..iterations {
        for material in &materials {
            let _ = generate_material_glsl(material, &GlslOptions);
        }
    }
    println!("material glsl x{}: {:?}", iterations, start.elapsed());

    start = Instant::now();
    for _ in 0..iterations {
        for effect in &post_effects {
            let _ = generate_post_wgsl(effect, &WgslOptions::default());
        }
    }
    println!("post wgsl x{}: {:?}", iterations, start.elapsed());

    start = Instant::now();
    for _ in 0..iterations {
        for material in &materials {
            let _ = material.pipeline_key().feature_bits;
        }
    }
    println!("pipeline_key x{}: {:?}", iterations, start.elapsed());

    println!(
        "\nscenix-nodes benchmark summary:\n  deep graphs: {}\n  fan graph nodes: {}\n  post effects: {}",
        deep.len(),
        fan.len(),
        post_effects.len()
    );
}
