//! scenix v1.6.0 "Shader Nodes And Node Materials" — facade-level CPU
//! coverage of the node material surface.

use scenix::{
    AlphaMode, Color, LightingStyle, Material, NodeDiagnosticKind, NodeInput, NodeMaterial,
    NodeValueKind, PostNodeEffect, ShaderGraph, ShaderKind, ShaderNodeKind, Stage, ToneMapKind,
    UniformInput, WgslOptions, generate_material_glsl, generate_material_wgsl,
};

fn lit_graph(base_pulsed_by_time: bool) -> ShaderGraph {
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let mut head = base;
    if base_pulsed_by_time {
        let time = graph.push(ShaderNodeKind::Uniform(UniformInput::Time));
        head = graph.add(ShaderNodeKind::Multiply, vec![node(base), node(time)]);
    }
    let lit = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), head);
    graph.add_fragment_output(lit, None);
    graph
}

fn node(id: scenix::ShaderNodeId) -> NodeInput {
    NodeInput::Node(id)
}

#[test]
fn node_material_lifecycle_exposes_cpu_state_and_custom_key() {
    let material = NodeMaterial::new("brick", lit_graph(false))
        .with_base_color(Color::from_rgb(0.7, 0.2, 0.1))
        .with_emissive(crate_vec3())
        .with_metallic(0.15)
        .with_roughness(0.9)
        .with_alpha_mode(AlphaMode::Blend)
        .with_double_sided(true);

    assert_eq!(material.name(), "brick");
    assert_eq!(material.base_color(), Color::from_rgb(0.7, 0.2, 0.1));
    assert_eq!(material.metallic(), 0.15);
    assert_eq!(material.roughness(), 0.9);
    assert_eq!(material.alpha_mode(), AlphaMode::Blend);
    assert!(material.double_sided());
    assert!(material.validate().is_empty());
    assert!(matches!(
        material.pipeline_key().shader,
        ShaderKind::Custom(_)
    ));

    // Spatially testing: `Some(cutoff)` selects masked alpha by contract.
    let masked = material.clone().with_alpha_cutoff(Some(0.25));
    assert_eq!(masked.alpha_mode(), AlphaMode::Mask(0.25));
    assert_eq!(masked.alpha_cutoff(), Some(0.25));

    // Live CPU state never changes the pipeline key.
    let hot = material
        .clone()
        .with_base_color(Color::from_rgb(1.0, 0.0, 0.0));
    assert_eq!(material.pipeline_key(), hot.pipeline_key());
}

#[test]
fn identical_graphs_share_the_pipeline_key() {
    let first = NodeMaterial::new("a", lit_graph(false));
    let second = NodeMaterial::new("b", lit_graph(false));
    assert_eq!(first.pipeline_key(), second.pipeline_key());
    assert_eq!(first.shader_hash(), second.shader_hash());

    let divergent = NodeMaterial::new("c", lit_graph(true));
    assert_ne!(first.shader_hash(), divergent.shader_hash());
}

#[test]
fn wgsl_output_matches_the_renderer_bind_contract() {
    let material = NodeMaterial::new("contract", lit_graph(true));
    let program = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
    let source = program.source();

    // Group order and light staging are renderer-fixed.
    assert!(source.contains("@group(0)"));
    assert!(source.contains("@group(1)"));
    assert!(source.contains("@group(2)"));
    assert!(source.contains("@group(3)"));
    // Time is the third frame-uniform slot.
    assert!(source.contains("frame.resolution.z"));
    // Fragment stage declares varyings matching VsOut.
    assert!(source.contains("v_normal"));
    assert!(source.contains("v_uv"));
    assert!(source.contains("v_world_position"));
    assert!(source.contains("v_tangent"));
    assert!(source.contains("v_position"));
    // The Lambert result is vec3, so alpha rides the vec4 constructor
    // (WGSL forbids vec3->vec4 casts); 1.0f is the explicit alpha default.
    assert!(source.contains("var final_color = vec4<f32>("));
    assert!(source.contains(", 1.0f);"));
}

#[test]
fn glsl_output_is_flattened_es300() {
    let material = NodeMaterial::new("flat", lit_graph(false));
    let glsl = generate_material_glsl(&material, &scenix::GlslOptions).unwrap();
    assert!(glsl.vertex_source().starts_with("#version 300 es\n"));
    assert!(glsl.fragment_source().contains("precision highp float"));
    assert!(
        glsl.fragment_source()
            .contains("uniform vec4 material_base_color")
    );
    // No uniform buffer blocks exist in the GLSL surface.
    assert!(!glsl.fragment_source().contains("uniform MaterialUniform"));
}

#[test]
fn displacement_port_reaches_both_backends() {
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    graph.add_fragment_output(base, None);
    let world = graph.push(ShaderNodeKind::WorldPosition);
    let offset = graph.add(
        ShaderNodeKind::Multiply,
        vec![node(world), NodeInput::Value(scenix::NodeValue::float(0.1))],
    );
    graph.set_vertex_offset(Some(offset));

    let material = NodeMaterial::new("wobble", graph);
    assert!(material.validate().is_empty());
    let wgsl = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
    assert!(wgsl.source().contains("displaced = displaced +"));
    let glsl = generate_material_glsl(&material, &scenix::GlslOptions).unwrap();
    assert!(glsl.vertex_source().contains("displaced = displaced +"));
}

#[test]
fn common_subexpressions_and_dead_nodes_collapse() {
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let sat = graph.chain(ShaderNodeKind::Saturate, base);
    let sat2 = graph.chain(ShaderNodeKind::Saturate, base);
    let mixed = graph.add(
        ShaderNodeKind::Mix,
        vec![node(sat), node(sat2), NodeInput::Value(value())],
    );
    graph.add_fragment_output(mixed, None);

    let material = NodeMaterial::new("merge", graph);
    let program = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
    assert_eq!(
        program
            .source()
            .matches("clamp(n0, vec4<f32>(0.0), vec4<f32>(1.0))")
            .count(),
        1,
        "two saturate nodes over the same input must collapse"
    );
}

#[test]
fn stage_compatibility_is_reported_with_location() {
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    graph.add_fragment_output(base, None);
    let screen_uv = graph.push(ShaderNodeKind::ScreenUv);
    let offset = graph.chain(ShaderNodeKind::Splat(NodeValueKind::Vec3), screen_uv);
    graph.set_vertex_offset(Some(offset));

    // Material validation is fragment-stage; the vertex gate is explicit.
    let vertex_diagnostics = graph.validate(Stage::Vertex);
    assert!(
        vertex_diagnostics.iter().any(|diagnostic| matches!(
            diagnostic.kind,
            NodeDiagnosticKind::StageUnsupported { .. }
        ))
    );

    // Uniforms are vertex/fragment only; post graphs reject them.
    let mut post_graph = ShaderGraph::new();
    let time = post_graph.push(ShaderNodeKind::Uniform(UniformInput::Time));
    post_graph.add_fragment_output(time, None);
    let effect = PostNodeEffect::new("uniform-in-post", post_graph);
    assert!(matches!(
        effect.validate().first().map(|d| &d.kind),
        Some(NodeDiagnosticKind::StageUnsupported { .. })
    ));
}

fn value() -> scenix::NodeValue {
    scenix::NodeValue::float(0.5)
}

#[test]
fn post_presets_compile_against_the_pass_contract() {
    let vignette = PostNodeEffect::vignette();
    let program = generate_post_wgsl_program(&vignette);
    assert!(program.contains("@group(0) @binding(0)"));
    assert!(program.contains("struct PostParams"));

    for preset in [
        PostNodeEffect::passthrough(),
        PostNodeEffect::desaturate(),
        PostNodeEffect::pixelate(),
        PostNodeEffect::color_grade(),
        PostNodeEffect::exposure(1.5),
    ] {
        assert!(preset.validate().is_empty());
        assert!(generate_post_wgsl_program(&preset).contains("fn fs_main("));
    }
}

#[test]
fn tonemap_accepts_rgba_base_colors() {
    // Color-like nodes accept rgba base colors; alpha is dropped in codegen.
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let tonemapped = graph.chain(ShaderNodeKind::ToneMap(ToneMapKind::Reinhard), base);
    graph.add_fragment_output(tonemapped, None);
    let material = NodeMaterial::new("graded", graph);
    assert!(material.validate().is_empty(), "rgba inputs must pass");
}

#[test]
fn facade_exports_carry_the_full_node_surface() {
    // Representative paths through the facade glob re-exports.
    let _: scenix::NodeMaterial = NodeMaterial::new("smoke", lit_graph(false));
    assert!(matches!(Stage::Vertex, scenix::Stage::Vertex));
    assert_eq!(scenix::slots::VIGNETTE_STRENGTH, 0);
    assert!(matches!(
        scenix::ShaderNodeKind::Uniform(UniformInput::Time),
        scenix::ShaderNodeKind::Uniform(UniformInput::Time)
    ));
    let _: scenix::ShaderNodeId = scenix::ShaderGraph::new().push(
        scenix::ShaderNodeKind::Attribute(scenix_nodes::VertexAttribute::Uv),
    );
}

fn crate_vec3() -> scenix::Vec3 {
    scenix::Vec3::new(0.2, 0.4, 0.6)
}

fn generate_post_wgsl_program(effect: &PostNodeEffect) -> String {
    scenix::generate_post_wgsl(effect, &WgslOptions::default())
        .unwrap()
        .source()
        .to_string()
}
