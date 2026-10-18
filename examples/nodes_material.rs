use scenix::{
    AlphaMode, Color, FEATURE_ALBEDO_TEXTURE, FEATURE_NODE_GRAPH, LightingStyle, Material,
    NodeInput, NodeMaterial, ShaderGraph, ShaderKind, ShaderNodeKind, TextureId, UniformInput,
};

fn main() {
    // A small lit graph: base color pulses over time and feeds a Lambert
    // term before reaching the fragment color port.
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let time = graph.push(ShaderNodeKind::Uniform(UniformInput::Time));
    let throb = graph.add(ShaderNodeKind::Multiply, vec![node(base), node(time)]);
    let lit = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), throb);
    graph.add_fragment_output(lit, None);

    let material = NodeMaterial::new("throb", graph)
        .with_base_color(Color::from_rgb(0.9, 0.4, 0.1))
        .with_metallic(0.0)
        .with_roughness(0.8)
        .with_alpha_mode(AlphaMode::Opaque);

    assert!(material.validate().is_empty());
    assert!(material.feature_bits() & FEATURE_NODE_GRAPH != 0);
    assert!(matches!(
        material.pipeline_key().shader,
        ShaderKind::Custom(_)
    ));
    println!(
        "node material `{}`: graph hash {:016x}, features {:#b}",
        material.name(),
        material.shader_hash(),
        material.feature_bits()
    );

    // A textured variant samples the registered albedo texture through the
    // mesh uv attribute.
    let mut uv_graph = ShaderGraph::new();
    let uv = uv_graph.push(ShaderNodeKind::Attribute(scenix_nodes::VertexAttribute::Uv));
    let sample = uv_graph.chain(ShaderNodeKind::TextureSample, uv);
    uv_graph.add_fragment_output(sample, None);
    let textured = NodeMaterial::new("textured", uv_graph)
        .with_texture(Some(TextureId::new(7)))
        .with_alpha_mode(AlphaMode::Mask(0.5));
    assert!(textured.validate().is_empty());
    assert!(textured.feature_bits() & FEATURE_ALBEDO_TEXTURE != 0);
    assert!(textured.feature_bits() & FEATURE_NODE_GRAPH != 0);
    assert!(textured.alpha_cutoff().is_some());
    println!(
        "textured node material `{}`: alpha cutoff {:?}, features {:#b}",
        textured.name(),
        textured.alpha_cutoff(),
        textured.feature_bits()
    );

    // Live CPU state is independent from graph identity: recoloring keeps
    // the same pipeline key.
    let recolored = material
        .clone()
        .with_base_color(Color::from_rgb(0.1, 0.6, 0.9));
    assert_ne!(material.base_color(), recolored.base_color());
    assert_eq!(material.pipeline_key(), recolored.pipeline_key());
    println!("recolored variant keeps the pipeline key");
}

/// Wraps a node handle as an input port connection.
fn node(id: scenix::ShaderNodeId) -> NodeInput {
    NodeInput::Node(id)
}
