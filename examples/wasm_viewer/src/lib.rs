use wasm_bindgen::prelude::*;

/// Creates the generated-scene scenix browser renderer.
#[wasm_bindgen]
pub async fn start(canvas: web_sys::HtmlCanvasElement) -> Result<scenix::WebRenderer, JsValue> {
    scenix::set_panic_hook();
    scenix::WebRenderer::new(canvas).await
}
