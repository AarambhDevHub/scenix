use leptos::prelude::*;

#[component]
pub fn FallbackPanel(status: RwSignal<String>) -> impl IntoView {
    view! {
        <div
            class="fallback-panel"
            class:hidden=move || status.get() == "WebGPU demo running"
            aria-live="polite"
        >
            <strong>{move || status.get()}</strong>
            <span>"The live WebGPU path is unavailable here, so this canvas is using a lightweight animated fallback."</span>
        </div>
    }
}
