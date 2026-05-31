# WASM And Browser

## Purpose

Use the browser wrapper, DOM input mapping, generated scene demo, and browser backend fallback.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Enable `wasm`. `scenix-wasm::BrowserRenderer` tries WebGPU where it is safe, uses WebGL when WebGPU is missing or unsuitable, and lets applications fall back to their own Canvas2D preview when WebGL is unavailable.

## Key Rules

- `scenix-wasm` wraps canvas setup and input forwarding.
- `WebRenderer` is the direct WebGPU path.
- `WebGlRenderer` is the direct WebGL compatibility path.
- `BrowserRenderer` chooses WebGPU first on safe browsers and WebGL otherwise.
- The website is Leptos CSR and builds with Trunk.
- Fallback UI should handle unavailable WebGPU and WebGL cleanly.

## Minimal Browser Renderer

```rust
use scenix_wasm::BrowserRenderer;
use wasm_bindgen::JsCast;

# async fn start() -> Result<(), wasm_bindgen::JsValue> {
let canvas = web_sys::window()
    .unwrap()
    .document()
    .unwrap()
    .get_element_by_id("scenix-canvas")
    .unwrap()
    .dyn_into::<web_sys::HtmlCanvasElement>()?;

let mut renderer = BrowserRenderer::new(canvas).await?;
renderer.tick(0.0)?;
web_sys::console::log_1(&renderer.backend_label().into());
# Ok(())
# }
```


## Example

```sh
rustup target add wasm32-unknown-unknown
cargo check -p scenix-wasm --target wasm32-unknown-unknown --all-features
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
