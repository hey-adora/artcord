use crate::app::components::navbar::Navbar;
use leptos::*;

#[component]
pub fn Login() -> impl IntoView {
    view! {
        <main class=move||format!("grid grid-rows-[1fr] min-h-[100dvh] transition-all duration-300 pt-[4rem]")>
            <Navbar/>
            <section>
                "Login"
            </section>
        </main>
    }
}
