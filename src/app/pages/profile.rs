use leptos::*;

use crate::app::components::navbar::Navbar;

#[component]
pub fn Profile() -> impl IntoView {
    view! {
        <main>
            <Navbar/>
            <section class="pt-[4rem] md:pt-[6rem]">
                "test8"
            </section>
        </main>
    }
}
