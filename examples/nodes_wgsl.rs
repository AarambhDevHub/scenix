use scenix::{
    LightingStyle, NodeInput, NodeMaterial, NodeValue, ShaderGraph, ShaderNodeKind, WgslOptions,
    generate_material_glsl, generate_material_wgsl,
};

fn main() {
    // One graph exercising lighting, texture sampling, and vertex
    // displacement through the optional displacement port.
    let mut graph = ShaderGraph::new();
    let uv = graph.push(ShaderNodeKind::Attribute(scenix_nodes::VertexAttribute::Uv));
    let sample = graph.chain(ShaderNodeKind::TextureSample, uv);
    let world_normal = graph.push(ShaderNodeKind::WorldNormal);
    let lit = graph.add(
        ShaderNodeKind::Multiply,
        vec![node(sample), node(world_normal)],
    );
    let lambert = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), lit);
    graph.add_fragment_output(lambert, None);

    // Displacement: world position pushed outward by a constant 0.1.
    let world_pos = graph.push(ShaderNodeKind::WorldPosition);
    let displacement = graph.add(
        ShaderNodeKind::Multiply,
        vec![node(world_pos), value(NodeValue::float(0.1))],
    );
    graph.set_vertex_offset(Some(displacement));

    let material = NodeMaterial::new("displaced", graph);
    assert!(material.validate().is_empty());

    // WGSL backend — the renderer contract: group 2 is the material group,
    // time uniform resolves to frame.resolution.z, and displacement lands
    // on the vertex output's local position field.
    let wgsl = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
    let source = wgsl.source();
    assert!(source.contains("@group(2)"));
    assert!(source.contains("struct MaterialUniform"));
    assert!(source.contains("displaced = displaced +"));
    println!("--- node-material WGSL ---\n{source}\n");

    // GLSL backend for WebGL tooling: same statements, ES300 surface.
    let glsl = generate_material_glsl(&material, &scenix::GlslOptions).unwrap();
    assert!(glsl.vertex_source().contains("displaced = displaced +"));
    assert!(glsl.fragment_source().contains("#version 300 es"));
    println!(
        "--- node-material GLSL (fragment) ---\n{}",
        glsl.fragment_source()
    );
}

/// Wraps a node handle as an input port connection.
fn node(id: scenix::ShaderNodeId) -> NodeInput {
    NodeInput::Node(id)
}

/// Wraps a raw constant as an input port connection.
fn value(value: NodeValue) -> NodeInput {
    NodeInput::Value(value)
}
