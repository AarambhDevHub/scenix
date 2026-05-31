# `scenix-wasm`

## Role

Optional browser canvas wrapper, DOM input mapping, generated scene setup, WebGPU/WebGL backend selection, and demo state getters.

## Dependency Weight

Browser-oriented optional path; enable `wasm` on facade. This pulls browser bindings, `scenix-renderer` for WebGPU, and lightweight WebGL code for compatibility fallback.

## Install

```toml
[dependencies]
scenix-wasm = "1"
```

## Key Public API

BrowserRenderer, BrowserBackendPreference, BrowserBackendKind, WebRenderer, WebGlRenderer, set_panic_hook, key_code_from_dom, pointer_button_from_dom, canvas_size

## Backend Choice

Use `BrowserRenderer` for applications and demos that should work across browsers:

```rust
use scenix_wasm::BrowserRenderer;
use wasm_bindgen::JsCast;

# async fn run() -> Result<(), wasm_bindgen::JsValue> {
let canvas = web_sys::window()
    .unwrap()
    .document()
    .unwrap()
    .get_element_by_id("canvas")
    .unwrap()
    .dyn_into::<web_sys::HtmlCanvasElement>()?;

let mut renderer = BrowserRenderer::new(canvas).await?;
renderer.tick(0.0)?;
let active_backend = renderer.backend_label();
# let _ = active_backend;
# Ok(())
# }
```

Use `WebRenderer` only when you require WebGPU specifically. Use `WebGlRenderer` to force the compatibility path in browser tests or product demos.

## Common Use

```sh
cargo check -p scenix-wasm --target wasm32-unknown-unknown --all-features
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
