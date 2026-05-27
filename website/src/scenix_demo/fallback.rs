use leptos::prelude::*;

#[component]
pub fn FallbackPanel(status: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="fallback-panel" aria-live="polite">
            <strong>{move || status.get()}</strong>
            <span>"A browser with WebGPU and WebAssembly support is required for the live canvas."</span>
        </div>
    }
}
