use leptos::prelude::*;

#[component]
pub fn Hero() -> impl IntoView {
    view! {
        <section class="hero" id="top">
            <div class="hero-copy">
                <p class="eyebrow">"Rust-native 3D workspace"</p>
                <h1>"Scenix"</h1>
                <p class="subtitle">"Modular Rust-native 3D scenes for native and WASM apps."</p>
                <div class="hero-actions" aria-label="Project links">
                    <a class="button primary" href="https://github.com/AarambhDevHub/scenix">"GitHub"</a>
                    <a class="button" href="https://crates.io/crates/scenix">"crates.io"</a>
                    <a class="button" href="https://docs.rs/scenix">"docs.rs"</a>
                    <a class="button" href="#demo">"Live Demo"</a>
                </div>
                <code class="install-command">"cargo add scenix"</code>
            </div>
            <div class="hero-visual" aria-hidden="true">
                <div class="stage-grid"></div>
                <div class="shape cube"></div>
                <div class="shape sphere"></div>
                <div class="shape torus"></div>
            </div>
        </section>
    }
}
