//! Standalone shader-node GLSL preflight for browser tooling (scenix
//! v1.6.0), behind the `nodes` feature.
//!
//! Browser users submit a serialized [`scenix_nodes::NodeMaterial`]; scenix
//! resolves the graph, generates both GLSL stages, and reports the first
//! diagnostic without any GPU or renderer state. This lets the node editor
//! surface compile results before anything is uploaded.

use wasm_bindgen::prelude::*;

use scenix_nodes::{NodeMaterial, generate_material_glsl};

/// Serialized `NodeMaterial` -> generated GLSL with a diagnostic, if any.
///
/// The returned [`NodeMaterialPreflight`] never throws: failures surface
/// through its `error` getter so the editor UI can render them inline.
#[wasm_bindgen(js_name = compileNodeMaterialGlsl)]
pub fn compile_node_material_glsl(material_json: &str) -> Result<NodeMaterialPreflight, JsValue> {
    let material: NodeMaterial = match serde_json::from_str(material_json) {
        Ok(material) => material,
        Err(error) => {
            return Ok(NodeMaterialPreflight::failed(format!(
                "invalid material json: {error}"
            )));
        }
    };
    Ok(match preflight(&material) {
        Ok(glsl) => NodeMaterialPreflight::compiled(glsl),
        Err(message) => NodeMaterialPreflight::failed(message),
    })
}

/// Validates a serialized `NodeMaterial` without generating shader code.
///
/// Returns `None` for a valid material, or the first diagnostic text.
#[wasm_bindgen(js_name = validateNodeMaterialJson)]
pub fn validate_node_material_json(material_json: &str) -> Result<Option<String>, JsValue> {
    let material: NodeMaterial = match serde_json::from_str(material_json) {
        Ok(material) => material,
        Err(error) => {
            return Err(JsValue::from_str(&format!(
                "invalid material json: {error}"
            )));
        }
    };
    let mut diagnostics = material.validate();
    if let Some(diagnostic) = diagnostics.pop() {
        return Ok(Some(diagnostic.to_string()));
    }
    Ok(None)
}

/// Resolves and compiles one material into (vertex, fragment) GLSL.
fn preflight(material: &NodeMaterial) -> Result<(String, String), String> {
    if !material.validate().is_empty() {
        let diagnostic = material.validate()[0].to_string();
        return Err(diagnostic);
    }
    let glsl = generate_material_glsl(material, &scenix_nodes::GlslOptions)
        .map_err(|diagnostic| diagnostic.to_string())?;
    Ok((
        glsl.vertex_source().to_owned(),
        glsl.fragment_source().to_owned(),
    ))
}

/// Preflight outcome exchanged with JavaScript.
#[wasm_bindgen]
pub struct NodeMaterialPreflight {
    vertex_source: String,
    fragment_source: String,
    error: Option<String>,
}

#[wasm_bindgen]
impl NodeMaterialPreflight {
    /// Generated vertex stage; empty when the preflight failed.
    #[wasm_bindgen(getter)]
    pub fn vertex_source(&self) -> String {
        self.vertex_source.clone()
    }

    /// Generated fragment stage; empty when the preflight failed.
    #[wasm_bindgen(getter)]
    pub fn fragment_source(&self) -> String {
        self.fragment_source.clone()
    }

    /// Whether both stages generated cleanly.
    #[wasm_bindgen(getter, js_name = ok)]
    pub fn ok(&self) -> bool {
        self.error.is_none()
    }

    /// First diagnostic text, or `undefined` for a clean compile.
    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}

impl NodeMaterialPreflight {
    /// Successful preflight payload.
    fn compiled(sources: (String, String)) -> Self {
        Self {
            vertex_source: sources.0,
            fragment_source: sources.1,
            error: None,
        }
    }

    /// Failed preflight payload with an empty stage pair.
    fn failed(message: String) -> Self {
        Self {
            vertex_source: String::new(),
            fragment_source: String::new(),
            error: Some(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenix_nodes::{ShaderGraph, ShaderNodeKind, UniformInput};

    fn material_json() -> String {
        let material = NodeMaterial::new("preflight", passthrough_graph());
        serde_json::to_string(&material).unwrap()
    }

    fn passthrough_graph() -> ShaderGraph {
        let mut graph = ShaderGraph::new();
        let source = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
        graph.add_fragment_output(source, None);
        graph
    }

    #[test]
    fn clean_json_compiles_both_stages() {
        let preflight = compile_node_material_glsl(&material_json()).unwrap();
        assert!(preflight.ok());
        assert!(preflight.vertex_source().contains("gl_Position"));
        assert!(preflight.fragment_source().contains("out vec4 frag_color"));
    }

    #[test]
    fn invalid_json_reports_parse_failure() {
        let preflight = compile_node_material_glsl("{not json").unwrap();
        assert!(!preflight.ok());
        assert!(preflight.error().unwrap().contains("invalid material json"));
    }

    #[test]
    fn broken_graph_reports_port_diagnostics() {
        // ScreenUv (Vec2) feeding Saturate expects a float or matching shape;
        // the Vec2 -> vec4 saturate mismatch must surface as a diagnostic.
        let mut graph = ShaderGraph::new();
        let screen_uv = graph.push(ShaderNodeKind::ScreenUv);
        let flat = graph.chain(
            ShaderNodeKind::Splat(scenix_nodes::NodeValueKind::Float),
            screen_uv,
        );
        graph.add_fragment_output(flat, None);
        let material = NodeMaterial::new("broken", graph);
        let json = serde_json::to_string(&material).unwrap();
        let preflight = compile_node_material_glsl(&json).unwrap();
        assert!(!preflight.ok());
        assert!(preflight.error().is_some());
    }
}
