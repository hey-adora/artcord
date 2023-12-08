use leptos::*;

use crate::app::components::navbar::Navbar;

#[component]
pub fn Profile() -> impl IntoView {
    view! {
        <main>
            <Navbar/>
            <section>
                "test"
            </section>
        </main>
    }
}
