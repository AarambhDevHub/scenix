use leptos::prelude::*;

#[component]
pub fn Hero() -> impl IntoView {
    view! {
        <section class="hero" id="top">
            <div class="hero-copy">
                <div class="hero-kicker-row">
                    <p class="eyebrow">"Rust-native 3D workspace"</p>
                    <span class="version-pill">"v1.0 stable"</span>
                </div>
                <h1>"Scenix"</h1>
                <p class="subtitle">"Modular Rust-native 3D scenes for native and WASM apps."</p>
                <div class="hero-actions" aria-label="Project links">
                    <a class="button primary" href="https://github.com/AarambhDevHub/scenix">"GitHub"</a>
                    <a class="button" href="https://crates.io/crates/scenix">"crates.io"</a>
                    <a class="button" href="https://docs.rs/scenix">"docs.rs"</a>
                    <a class="button" href="#demo">"Live Demo"</a>
                </div>
                <code class="install-command">"cargo add scenix"</code>
                <div class="hero-metrics" aria-label="Scenix release metrics">
                    <div>
                        <strong>"17"</strong>
                        <span>"modular crates"</span>
                    </div>
                    <div>
                        <strong>"CPU"</strong>
                        <span>"default facade"</span>
                    </div>
                    <div>
                        <strong>"WASM"</strong>
                        <span>"optional demo"</span>
                    </div>
                </div>
            </div>
            <div class="hero-visual" aria-hidden="true">
                <div class="hero-panel-top">
                    <span>"orbit camera"</span>
                    <span>"raycast pick"</span>
                    <span>"animato tick"</span>
                </div>
                <div class="hero-horizon"></div>
                <div class="stage-grid"></div>
                <div class="hero-rover">
                    <div class="rover-shadow"></div>
                    <div class="rover-wheel wheel-left"></div>
                    <div class="rover-wheel wheel-right"></div>
                    <div class="rover-body">
                        <span class="body-panel panel-teal"></span>
                        <span class="body-panel panel-gold"></span>
                        <span class="body-panel panel-violet"></span>
                    </div>
                    <div class="rover-cabin"></div>
                    <div class="rover-arm"></div>
                    <div class="rover-tool"></div>
                    <div class="scan-ring"></div>
                </div>
                <div class="hero-light"></div>
                <div class="hero-node node-a"></div>
                <div class="hero-node node-b"></div>
                <div class="hero-node node-c"></div>
                <div class="hero-inspector">
                    <span>"selected"</span>
                    <strong>"Scenix Rover"</strong>
                    <small>"PBR teal alloy · helpers on"</small>
                </div>
            </div>
        </section>
    }
}
