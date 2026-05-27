use leptos::prelude::*;

use crate::components::{
    code_panel::CodePanel, crate_map::CrateMap, demo_panel::DemoPanel, examples::Examples,
    feature_grid::FeatureGrid, footer::Footer, hero::Hero, navbar::Navbar,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Navbar />
        <main>
            <Hero />
            <DemoPanel />
            <FeatureGrid />
            <CrateMap />
            <CodePanel />
            <Examples />
            <section class="release-section" id="release">
                <div>
                    <p class="eyebrow">"Stable Release"</p>
                    <h2>"v1.0.0 Stable"</h2>
                    <p>
                        "API freeze, complete docs, compiling examples, green CI, and a GitHub Pages demo built from the same modular crates."
                    </p>
                </div>
                <ul class="release-list">
                    <li>"Stable facade and crate APIs"</li>
                    <li>"Optional renderer, loader, post, Animato, and WASM features"</li>
                    <li>"Docs and release notes published with the crate"</li>
                    <li>"Static Leptos CSR website deployed under /scenix/"</li>
                </ul>
            </section>
        </main>
        <Footer />
    }
}
