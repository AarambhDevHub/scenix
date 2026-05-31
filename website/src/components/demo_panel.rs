use leptos::prelude::*;

use crate::scenix_demo::{bridge, controls::DemoControls, fallback::FallbackPanel};

#[component]
pub fn DemoPanel() -> impl IntoView {
    let playing = RwSignal::new(true);
    let helpers = RwSignal::new(true);
    let wireframe = RwSignal::new(false);
    let bloom = RwSignal::new(false);
    let ssao = RwSignal::new(false);
    let status = RwSignal::new(String::from("Starting browser demo"));
    let fps = RwSignal::new(String::from("0"));
    let selected = RwSignal::new(String::from("None"));
    let selected_id = RwSignal::new(String::from("0"));
    let distance = RwSignal::new(String::from("0.00"));
    let material = RwSignal::new(String::from("None"));
    let flags = RwSignal::new(String::from("helpers=true, raycaster=true, animato=true"));

    Effect::new(move |_| {
        bridge::start("scenix-canvas");
        bridge::start_snapshot_loop(move || {
            let snapshot = bridge::snapshot();
            status.set(snapshot.status);
            fps.set(format!("{:.0}", snapshot.fps));
            selected.set(snapshot.selected_name);
            selected_id.set(snapshot.selected_id.to_string());
            distance.set(format!("{:.2}", snapshot.distance));
            material.set(snapshot.material);
            flags.set(snapshot.flags);
        });
    });

    view! {
        <section class="demo-section" id="demo">
            <div class="demo-shell">
                <div class="canvas-wrap">
                    <div class="canvas-toolbar" aria-hidden="true">
                        <span>"Scenix Engine Lab"</span>
                        <span>"interactive preview"</span>
                    </div>
                    <canvas id="scenix-canvas" aria-label="Scenix Engine Lab live 3D demo"></canvas>
                    <FallbackPanel status=status />
                </div>
                <aside class="demo-side">
                    <div class="demo-status-row">
                        <p class="eyebrow">"Live 3D Demo"</p>
                        <span class="status-chip">{move || status.get()}</span>
                    </div>
                    <h2>"Scenix Engine Lab"</h2>
                    <p class="demo-side-copy">
                        "Drive the scene controls, pick objects in the canvas, and switch debug views without leaving the page."
                    </p>
                    <DemoControls playing helpers wireframe bloom ssao />
                    <dl class="stats">
                        <div><dt>"FPS"</dt><dd>{move || fps.get()}</dd></div>
                        <div><dt>"Selected"</dt><dd>{move || selected.get()}</dd></div>
                        <div><dt>"NodeId"</dt><dd>{move || selected_id.get()}</dd></div>
                        <div><dt>"Distance"</dt><dd>{move || distance.get()}</dd></div>
                        <div><dt>"Material"</dt><dd>{move || material.get()}</dd></div>
                        <div><dt>"Flags"</dt><dd>{move || flags.get()}</dd></div>
                    </dl>
                </aside>
            </div>
        </section>
    }
}
