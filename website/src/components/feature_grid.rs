use leptos::prelude::*;

const FEATURES: &[(&str, &str)] = &[
    ("Modular workspace", "Use only the crates your app needs."),
    ("Renderer-agnostic scene graph", "Author scenes without pulling GPU dependencies."),
    ("wgpu renderer", "Native and browser rendering through WebGPU backends."),
    ("Animato bridge", "Drive transforms, cameras, and materials with Animato 1.5."),
    ("WASM support", "Browser canvas wrapper with DOM input forwarding."),
    ("BVH raycasting", "Fast CPU picking with exact mesh intersections."),
    ("Debug helpers", "Grid, axes, bounds, camera, light, and skeleton lines."),
    ("Post-processing", "Bloom, SSAO, tonemap, AA, fog, outline, and blur stack."),
    ("no_std CPU crates", "Core authoring crates stay lightweight and portable."),
];

#[component]
pub fn FeatureGrid() -> impl IntoView {
    view! {
        <section class="section" id="why">
            <p class="eyebrow">"Why Scenix"</p>
            <h2>"A complete scene stack without a monolith"</h2>
            <div class="feature-grid">
                {FEATURES.iter().map(|(title, body)| view! {
                    <article class="feature-card">
                        <h3>{*title}</h3>
                        <p>{*body}</p>
                    </article>
                }).collect_view()}
            </div>
        </section>
    }
}
