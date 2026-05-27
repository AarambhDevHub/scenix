use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <span>"Aarambh Dev Hub"</span>
            <nav aria-label="Footer links">
                <a href="https://github.com/AarambhDevHub/scenix">"GitHub"</a>
                <a href="https://crates.io/crates/scenix">"crates.io"</a>
                <a href="https://docs.rs/scenix">"docs.rs"</a>
                <span>"MIT OR Apache-2.0"</span>
            </nav>
        </footer>
    }
}
