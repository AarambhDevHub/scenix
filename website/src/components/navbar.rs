use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <header class="navbar">
            <a class="brand" href="#top" aria-label="Scenix home">
                <span class="brand-mark"></span>
                <span>"Scenix"</span>
            </a>
            <nav aria-label="Primary navigation">
                <a href="#demo">"Demo"</a>
                <a href="#crates">"Crates"</a>
                <a href="#code">"Code"</a>
                <a href="#examples">"Examples"</a>
                <a class="nav-pill" href="#release">"v1.0"</a>
            </nav>
        </header>
    }
}
