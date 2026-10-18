//! Backend contract tests for the scenix v1.6.0 shader node system.
//!
//! These tests pin the lowering contracts exercised by the renderer and the
//! WASM facade: WGSL module layout, GLSL mirror sources, graph validation
//! diagnostics, dead-node pruning, CSE, deterministic hashing, and the post
//! pass contract.

use scenix_material::{AlphaMode, FEATURE_NODE_GRAPH, Material as _, ShaderKind};
use scenix_math::Vec4;
use scenix_nodes::glsl::{GlslOptions, generate_material_glsl, generate_post_glsl};
use scenix_nodes::wgsl::{WgslOptions, generate_material_wgsl, generate_post_wgsl};
use scenix_nodes::{
    ComponentIndex, LightingStyle, NodeDiagnosticKind, NodeInput, NodeMaterial, NodeValue,
    NodeValueKind, PostNodeEffect, ShaderGraph, ShaderNodeId, ShaderNodeKind, Stage, ToneMapKind,
    UniformInput,
};

fn node_input(id: ShaderNodeId) -> NodeInput {
    NodeInput::Node(id)
}

fn literal(value: NodeValue) -> NodeInput {
    NodeInput::Value(value)
}

fn base_lit_graph() -> ShaderGraph {
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let lit = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), base);
    graph.add_fragment_output(lit, None);
    graph
}

fn textured_graph() -> ShaderGraph {
    let mut graph = ShaderGraph::new();
    let uv = graph.push(ShaderNodeKind::ScreenUv);
    let sample = graph.chain(ShaderNodeKind::TextureSample, uv);
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let mixed = graph.add(
        ShaderNodeKind::Multiply,
        vec![node_input(sample), node_input(base)],
    );
    graph.add_fragment_output(mixed, None);
    graph
}

fn displaced_graph() -> ShaderGraph {
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let lit = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), base);
    graph.add_fragment_output(lit, None);
    // Vertex displacement: time * constant direction -> local offset.
    let time = graph.push(ShaderNodeKind::Uniform(UniformInput::Time));
    let direction = graph.push(ShaderNodeKind::Constant(NodeValue::rgb(0.0, 1.0, 0.0)));
    let offset = graph.add(
        ShaderNodeKind::Multiply,
        vec![node_input(time), node_input(direction)],
    );
    graph.set_vertex_offset(Some(offset));
    graph
}

fn first_diagnostic(graph: &ShaderGraph, stage: Stage) -> NodeDiagnosticKind {
    let diagnostics = graph.validate(stage);
    assert!(!diagnostics.is_empty(), "expected at least one diagnostic");
    diagnostics.into_iter().next().expect("checked").kind
}

#[test]
fn material_wgsl_matches_the_renderer_bind_contract() {
    let material = NodeMaterial::new("lit", base_lit_graph());
    assert!(material.validate().is_empty());
    assert_eq!(material.feature_bits(), FEATURE_NODE_GRAPH);

    let program = generate_material_wgsl(&material, &WgslOptions::default()).expect("valid graph");
    let source = program.source();

    assert!(source.contains("@vertex\nfn vs_main("));
    assert!(source.contains("@fragment\nfn fs_main("));
    assert_eq!(
        source
            .matches("@group(0) @binding(0) var<uniform> frame: FrameUniform;")
            .count(),
        1
    );
    assert!(source.contains("@group(2) @binding(2) var material_albedo: texture_2d<f32>;"));
    assert!(source.contains("@group(3) @binding(0) var<uniform> lights: LightUniform;"));
    assert!(source.contains("scenix_light_lambert"));
    // Vertex attribute locations mirror the PackedVertex layout (0..=4).
    assert!(source.contains("@location(4) tangent: vec4<f32>,"));
    // The lighting result is vec3, so alpha rides the vec4 constructor
    // (WGSL forbids vec3->vec4 casts); 1.0f is the default alpha port text.
    assert!(source.contains("var final_color = vec4<f32>("));
    assert!(source.contains(", 1.0f);"));
    assert!(source.contains("material.emissive_cutoff.a >= 0.0"));
}

#[test]
fn textured_graphs_sample_the_single_material_slot() {
    let material = NodeMaterial::new("textured", textured_graph()).with_texture(None);
    let program = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
    assert!(
        program
            .source()
            .contains("textureSample(material_albedo, material_sampler")
    );
    assert!(
        program
            .source()
            .contains("frag_coord.xy / frame.resolution.xy")
    );

    let glsl = generate_material_glsl(&material, &GlslOptions).unwrap();
    assert!(glsl.vertex_source().contains("#version 300 es"));
    assert!(glsl.fragment_source().contains("texture(material_albedo"));
    assert!(glsl.fragment_source().contains("in vec4 v_color;"));
    assert!(glsl.vertex_source().contains("out vec3 v_world_position;"));
}

#[test]
fn displacement_lowering_emits_the_local_vertex_port() {
    let material = NodeMaterial::new("bouncy", displaced_graph());
    assert!(material.validate().is_empty());
    let program = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
    assert!(program.source().contains("    displaced = displaced + n"));
    assert!(program.source().contains("out.local = displaced;"));
    assert!(program.source().contains("frame.resolution.z"));
}

#[test]
fn time_maps_to_the_elapsed_frame_resolution_slot() {
    // A bare f32 cannot feed the color port; multiply it into the base color.
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let time = graph.push(ShaderNodeKind::Uniform(UniformInput::Time));
    let shimmer = graph.add(
        ShaderNodeKind::Multiply,
        vec![node_input(base), node_input(time)],
    );
    graph.add_fragment_output(shimmer, None);
    let material = NodeMaterial::new("ticking", graph);
    let program = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
    assert!(program.source().contains("frame.resolution.z"));
}

#[test]
fn vec4_color_inputs_are_accepted_by_color_like_nodes() {
    // BaseColor is vec4; color-like nodes accept vec3 and vec4.
    let material = NodeMaterial::new("vec4ish", base_lit_graph());
    assert!(material.validate().is_empty());

    let glsl = generate_material_glsl(&material, &GlslOptions).unwrap();
    assert!(
        glsl.fragment_source().contains(".rgb, normalize(v_normal)"),
        "fragment source:\n{}",
        glsl.fragment_source()
    );
}

#[test]
fn common_subexpressions_merge() {
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let left = graph.chain(ShaderNodeKind::Saturate, base);
    let right = graph.chain(ShaderNodeKind::Saturate, base);
    let sum = graph.add(
        ShaderNodeKind::Add,
        vec![node_input(left), node_input(right)],
    );
    graph.add_fragment_output(sum, None);
    let material = NodeMaterial::new("merged", graph);
    let program = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
    // Both saturate nodes collapse into exactly one statement.
    assert_eq!(
        program
            .source()
            .matches("clamp(n0, vec4<f32>(0.0), vec4<f32>(1.0))")
            .count(),
        1,
        "source:\n{}",
        program.source()
    );
}

#[test]
fn dead_nodes_are_pruned_from_lowering() {
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let lit = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), base);
    graph.add_fragment_output(lit, None);
    // Orphan node referenced by no output port: dead code for lowering.
    let orphan = graph.chain(ShaderNodeKind::ToneMap(ToneMapKind::Aces), base);
    let _ = orphan;

    let material = NodeMaterial::new("pruned", graph.clone());
    assert!(material.validate().is_empty());

    let program = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
    assert!(!program.source().contains("scenix_tonemap_aces"));
    assert!(program.source().contains("scenix_light_lambert"));
    assert_eq!(graph.evaluation_order().len(), 2);
}

#[test]
fn diagnostics_pin_expected_failures() {
    // vec4 fed into a splat that expects a scalar.
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let splatted = graph.chain(ShaderNodeKind::Splat(NodeValueKind::Vec3), base);
    graph.add_fragment_output(splatted, None);
    assert!(matches!(
        first_diagnostic(&graph, Stage::Fragment),
        NodeDiagnosticKind::TypeMismatch { .. }
    ));

    // Component selector past the vector's channel count.
    let mut graph = ShaderGraph::new();
    let picked = graph.chain(
        ShaderNodeKind::Component(ComponentIndex::Z),
        literal(NodeValue::splat(NodeValueKind::Vec2, 0.5)),
    );
    graph.add_fragment_output(picked, None);
    assert!(matches!(
        first_diagnostic(&graph, Stage::Fragment),
        NodeDiagnosticKind::ComponentOutOfRange { .. }
    ));

    // Fragment color port type rule (f32 cannot feed the color output).
    let mut graph = ShaderGraph::new();
    let time = graph.push(ShaderNodeKind::Uniform(UniformInput::Time));
    graph.add_fragment_output(time, None);
    assert!(matches!(
        first_diagnostic(&graph, Stage::Fragment),
        NodeDiagnosticKind::OutputTypeMismatch { .. }
    ));

    // Missing input port.
    let mut graph = ShaderGraph::new();
    let mix = graph.add(ShaderNodeKind::Mix, vec![]);
    graph.add_fragment_output(mix, None);
    assert!(matches!(
        first_diagnostic(&graph, Stage::Fragment),
        NodeDiagnosticKind::MissingInput { .. }
    ));
}

#[test]
fn cycles_are_rejected() {
    let mut graph = ShaderGraph::new();
    let first = graph.add(ShaderNodeKind::OneMinus, vec![]);
    let second = graph.chain(ShaderNodeKind::OneMinus, first);
    // Close the loop in place: first -> second is known; add second -> first.
    graph
        .set_node_inputs(first, vec![node_input(second)])
        .expect("known node");
    graph.add_fragment_output(second, None);
    assert!(matches!(
        first_diagnostic(&graph, Stage::Fragment),
        NodeDiagnosticKind::Cycle(_)
    ));
}

#[test]
fn live_subgraph_size_is_capped() {
    let mut graph = ShaderGraph::new();
    let mut current = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    for _ in 0..scenix_nodes::NODE_GRAPH_LIMIT + 32 {
        let next = graph.chain(ShaderNodeKind::OneMinus, current);
        current = next;
    }
    graph.add_fragment_output(current, None);
    assert!(matches!(
        first_diagnostic(&graph, Stage::Fragment),
        NodeDiagnosticKind::GraphTooLarge { .. }
    ));
}

#[test]
fn vertex_stage_rules_shadow_fragment_features() {
    // ScreenUv is fragment-only; reachable through the vertex displacement
    // port it must surface as StageUnsupported during vertex validation.
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    graph.add_fragment_output(base, None);
    let screen_uv = graph.push(ShaderNodeKind::ScreenUv);
    let offset = graph.chain(ShaderNodeKind::Splat(NodeValueKind::Vec3), screen_uv);
    graph.set_vertex_offset(Some(offset));
    assert!(
        graph
            .validate(Stage::Vertex)
            .iter()
            .any(|diagnostic| matches!(
                diagnostic.kind,
                NodeDiagnosticKind::StageUnsupported { .. }
            )),
        "vertex diagnostics: {:?}",
        graph.validate(Stage::Vertex)
    );
}

#[test]
fn shader_hash_tracks_graph_identity() {
    let graph_a = base_lit_graph();
    let graph_b = base_lit_graph();
    assert_eq!(graph_a.shader_hash(), graph_b.shader_hash());

    let mut graph_c = base_lit_graph();
    graph_c.push(ShaderNodeKind::Uniform(UniformInput::Metallic));
    assert_ne!(graph_a.shader_hash(), graph_c.shader_hash());
}

#[test]
fn evaluation_order_is_postorder_over_live_nodes() {
    let graph = displaced_graph();
    let order = graph.evaluation_order();
    // Live nodes: base -> lambert plus time/direction/offset = 5.
    assert_eq!(order.len(), 5);
    // The postorder tail is the vertex-offset root (a Multiply).
    let last = order[order.len() - 1];
    let last_node = graph.get(last).expect("live root");
    assert!(matches!(last_node.kind, ShaderNodeKind::Multiply));

    // Every node's inputs appear earlier in the order.
    for (index, id) in order.iter().enumerate() {
        if let Some(node) = graph.get(*id) {
            for input in &node.inputs {
                if let NodeInput::Node(child) = input {
                    let position = order
                        .iter()
                        .position(|visited| visited == child)
                        .expect("child precedes parent");
                    assert!(position < index);
                }
            }
        }
    }
}

#[test]
fn post_effects_lower_to_the_post_contract() {
    for effect in [
        PostNodeEffect::passthrough(),
        PostNodeEffect::vignette(),
        PostNodeEffect::desaturate(),
        PostNodeEffect::posterize(),
        PostNodeEffect::pixelate(),
        PostNodeEffect::color_grade(),
        PostNodeEffect::exposure(1.25),
    ] {
        assert!(effect.validate().is_empty(), "{} invalid", effect.name());

        let wgsl = generate_post_wgsl(&effect, &WgslOptions::default()).unwrap();
        assert!(
            wgsl.source()
                .contains("@group(0) @binding(2) var<uniform> params: PostParams;")
        );
        assert!(
            wgsl.source()
                .contains("out.uv = position * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);")
        );

        let glsl = generate_post_glsl(&effect, &GlslOptions).unwrap();
        assert!(glsl.vertex_source().contains("gl_VertexID"));
        assert!(glsl.fragment_source().contains("uniform vec4 values0;"));
    }
}

#[test]
fn node_material_keys_use_the_custom_shader_family() {
    let material = NodeMaterial::new("keyed", base_lit_graph()).with_metallic(0.4);
    let key = material.pipeline_key();
    assert!(matches!(key.shader, ShaderKind::Custom(_)));
    assert!(key.feature_bits & FEATURE_NODE_GRAPH != 0);
    assert!(!material.is_transparent());

    let masked = material.clone().with_alpha_mode(AlphaMode::Mask(0.5));
    assert_eq!(masked.alpha_cutoff(), Some(0.5));

    let blended = material.clone().with_alpha_mode(AlphaMode::Blend);
    assert!(blended.is_transparent());
}

#[test]
fn root_exports_are_feature_free() {
    use scenix_nodes as root;
    let diagnostic =
        root::NodeDiagnostic::graph_level(root::NodeDiagnosticKind::MissingFragmentOutput);
    assert!(diagnostic.node.is_none());
    assert_ne!(root::Stage::Vertex, root::Stage::Post);
}

#[test]
fn luma_weights_sum_to_black() {
    let weights = Vec4::new(0.2126, 0.7152, 0.0722, 0.0);
    assert!((weights.x + weights.y + weights.z - 1.0).abs() < 1e-3);
}
