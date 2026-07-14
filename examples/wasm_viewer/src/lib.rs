use wasm_bindgen::prelude::*;

/// Creates the generated-scene renderer with WebGPU-first/WebGL fallback.
#[wasm_bindgen]
pub async fn start(canvas: web_sys::HtmlCanvasElement) -> Result<scenix::BrowserRenderer, JsValue> {
    scenix::set_panic_hook();
    scenix::BrowserRenderer::new(canvas).await
}
