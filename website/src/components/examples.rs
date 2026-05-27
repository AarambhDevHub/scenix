use leptos::prelude::*;

const EXAMPLES: &[(&str, &str)] = &[
    ("Hello Cube", "Headless cube render"),
    ("PBR Sphere", "Metallic roughness material"),
    ("GLTF Scene", "Generated glTF loading"),
    ("Raycasting", "BVH scene picking"),
    ("Animato Integration", "Node, camera, material tracks"),
    ("WASM Viewer", "Canvas renderer wrapper"),
    ("Post Processing", "SSAO, bloom, tonemap, FXAA, TAA"),
    ("Helpers Demo", "Grid, axes, bounds, lights"),
];

#[component]
pub fn Examples() -> impl IntoView {
    view! {
        <section class="section" id="examples">
            <p class="eyebrow">"Examples"</p>
            <h2>"Reference scenes that compile"</h2>
            <div class="example-grid">
                {EXAMPLES.iter().map(|(title, body)| view! {
                    <article class="example-card">
                        <h3>{*title}</h3>
                        <p>{*body}</p>
                    </article>
                }).collect_view()}
            </div>
        </section>
    }
}
