# Scenix v1.1.0 Browser Fallback

Scenix `1.1.0` improves browser reliability by adding a real WebGL fallback inside `scenix-wasm`. The stable v1 API remains additive: the existing WebGPU `WebRenderer` stays available, and applications can now use `BrowserRenderer` when they want automatic backend selection.

## Highlights

- All workspace crates are bumped to `1.1.0`.
- `scenix-wasm` now provides `BrowserRenderer`, `WebGlRenderer`, `BrowserBackendPreference`, and `BrowserBackendKind`.
- Browser startup now chooses WebGPU where safe, WebGL when WebGPU is unavailable or unsuitable, and leaves Canvas2D as the final website fallback.
- The website demo uses the crate-level `BrowserRenderer` instead of website-only WebGPU detection.
- WebGL renders the generated Scenix Engine Lab scene with depth testing, camera matrices, material color preview, helper visibility, wireframe preview, animation controls, and CPU raycast picking.
- Publish and Pages workflows build the website with a Trunk-compatible `NO_COLOR=false` environment.

## Install

```toml
[dependencies]
scenix = "1.1"
```

Browser support through the facade:

```toml
[dependencies]
scenix = { version = "1.1", features = ["wasm"] }
```

Optional full stack:

```toml
[dependencies]
scenix = { version = "1.1", features = ["loader", "renderer", "post", "animato", "wasm"] }
```

## What Changed From 1.0.0

- Bumped all workspace crates from `1.0.0` to `1.1.0`.
- Added a WebGL compatibility renderer inside `scenix-wasm`; no new crate is required.
- Added automatic browser backend selection through `BrowserRenderer`.
- Updated the website bridge so Firefox and browsers without usable WebGPU try WebGL before Canvas2D.
- Updated documentation, changelog, tests, and GitHub Release automation for the `v1.1.0` release.

## Code Example

```rust
use scenix_wasm::BrowserRenderer;
use wasm_bindgen::JsCast;

# async fn run() -> Result<(), wasm_bindgen::JsValue> {
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

## Migration Notes

- Existing `WebRenderer` users do not need to change code if they require WebGPU directly.
- Browser applications should prefer `BrowserRenderer` for production demos and websites because it handles WebGPU-to-WebGL fallback.
- Canvas2D fallback remains application-level UI behavior; it is not exposed as a Scenix rendering backend.
- Keep explicit feature flags for loader, renderer, post, Animato, and WASM paths.

## Known Limitations

- WebGL browser rendering is a compatibility preview path; it is not full `wgpu` feature parity.
- WebGL does not implement real shadows, post-processing, GPU texture upload parity, or physically accurate PBR.
- Loader APIs decode CPU assets but do not automatically upload them to either renderer.
- GPU tests still depend on a working Vulkan backend or Mesa lavapipe.

## Links

- Website and demo: `https://aarambhdevhub.github.io/scenix/`
- Documentation: `https://docs.rs/scenix`
- Crates: `https://crates.io/crates/scenix`
