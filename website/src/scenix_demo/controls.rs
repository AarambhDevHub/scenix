use leptos::prelude::*;

use crate::scenix_demo::bridge;

#[component]
pub fn DemoControls(
    playing: RwSignal<bool>,
    helpers: RwSignal<bool>,
    wireframe: RwSignal<bool>,
    bloom: RwSignal<bool>,
    ssao: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="controls" aria-label="Demo controls">
            <button type="button" on:click=move |_| {
                let next = !playing.get();
                playing.set(next);
                bridge::set_playing(next);
            }>{move || if playing.get() { "Pause" } else { "Play" }}</button>
            <button type="button" on:click=move |_| {
                let next = !helpers.get();
                helpers.set(next);
                bridge::set_helpers_visible(next);
            }>{move || if helpers.get() { "Hide Helpers" } else { "Show Helpers" }}</button>
            <button type="button" on:click=move |_| {
                let next = !wireframe.get();
                wireframe.set(next);
                bridge::set_wireframe_enabled(next);
            }>{move || if wireframe.get() { "Solid" } else { "Wireframe" }}</button>
            <button type="button" on:click=move |_| {
                let next = !bloom.get();
                bloom.set(next);
                bridge::set_bloom_enabled(next);
            }>{move || if bloom.get() { "Bloom On" } else { "Bloom Off" }}</button>
            <button type="button" on:click=move |_| {
                let next = !ssao.get();
                ssao.set(next);
                bridge::set_ssao_enabled(next);
            }>{move || if ssao.get() { "SSAO On" } else { "SSAO Off" }}</button>
            <button type="button" on:click=move |_| bridge::reset_camera()>"Reset Camera"</button>
        </div>
    }
}
