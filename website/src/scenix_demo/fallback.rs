use leptos::prelude::*;

#[component]
pub fn FallbackPanel(status: RwSignal<String>) -> impl IntoView {
    view! {
        <div
            class="fallback-panel"
            class:hidden=move || {
                let status = status.get();
                status == "webgpu demo running" || status == "webgl demo running"
            }
            aria-live="polite"
        >
            <strong>{move || status.get()}</strong>
            <span>"The live WebGPU/WebGL path is unavailable here, so this canvas is using a lightweight animated fallback."</span>
        </div>
    }
}
