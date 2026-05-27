use leptos::prelude::*;

const CODE: &str = r#"use scenix::*;

let mut scene = SceneGraph::new();
let cube = scene.add(SceneNode::mesh(
    "cube",
    MeshId::new(1),
    MaterialId::new(1),
));

let camera = PerspectiveCamera::new(60.0, 16.0 / 9.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 1.5, 4.0))
    .target(Vec3::ZERO);

let mut driver = ScenixAnimationDriver::new();
driver.add_node(NodeAnimator::new(
    cube,
    NodeAnimationTarget::Translation(Vec3Track::tween(
        Vec3::ZERO,
        Vec3::new(0.0, 0.5, 0.0),
        1.0,
    )),
));"#;

#[component]
pub fn CodePanel() -> impl IntoView {
    view! {
        <section class="section code-section" id="code">
            <div>
                <p class="eyebrow">"Code Example"</p>
                <h2>"Scene, camera, animation"</h2>
                <p>"The facade keeps common types in one import while renderer, loader, post, Animato, and WASM support remain opt-in features."</p>
            </div>
            <pre class="code-panel"><code>{CODE}</code></pre>
        </section>
    }
}
