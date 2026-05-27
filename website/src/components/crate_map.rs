use leptos::prelude::*;

const CRATES: &[&str] = &[
    "scenix",
    "scenix-math",
    "scenix-core",
    "scenix-scene",
    "scenix-camera",
    "scenix-mesh",
    "scenix-material",
    "scenix-light",
    "scenix-texture",
    "scenix-renderer",
    "scenix-loader",
    "scenix-post",
    "scenix-raycaster",
    "scenix-helpers",
    "scenix-animato",
    "scenix-wasm",
    "scenix-input",
];

#[component]
pub fn CrateMap() -> impl IntoView {
    view! {
        <section class="section" id="crates">
            <p class="eyebrow">"Crate Map"</p>
            <h2>"Composable by default"</h2>
            <div class="crate-map">
                {CRATES.iter().map(|name| view! {
                    <a href=format!("https://crates.io/crates/{name}")>{*name}</a>
                }).collect_view()}
            </div>
        </section>
    }
}
