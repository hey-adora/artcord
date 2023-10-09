use crate::app::utils::GlobalState;
use components::navbar::Navbar;
use leptos::html::{ElementDescriptor, ToHtmlElement};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use pages::gallery::GalleryPage;
use pages::home::HomePage;
use pages::not_found::NotFound;
use std::ops::Deref;

mod components;
mod pages;
mod utils;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(GlobalState::new());

    view! {
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        <Title text="Welcome to Leptos"/>
        <Body class="text-low-purple pt-4 bg-gradient-to-br from-mid-purple to-dark-purple" />

        <Router>
            <Navbar/>
            <main id="home" on:scroll=|_|{ logging::log!("SCROLLED!"); }  class=" grid grid-rows-[auto_1fr] pt-4 gap-6       ">
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/gallery" view=GalleryPage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}
