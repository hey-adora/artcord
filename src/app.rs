use leptos::logging::log;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::{use_websocket, UseWebsocketReturn};

use components::navbar::Navbar;
use pages::gallery::GalleryPage;
use pages::home::HomePage;
use pages::not_found::NotFound;

use crate::app::utils::GlobalState;

mod components;
mod pages;
mod utils;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(GlobalState::new());
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");

    create_effect(move |_| {
        let UseWebsocketReturn {
            ready_state,
            message,
            message_bytes,
            send,
            send_bytes,
            open,
            close,
            ..
        } = use_websocket("/ws/");
    });

    view! {
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        <Title text="Welcome to Leptos"/>
        <Body  class=move || format!("text-low-purple  bg-gradient-to-br from-mid-purple to-dark-purple   {}", if global_state.nav_open.get() == true { "overflow-hidden w-screen h-screen" } else { "" })  />
        <Router>
            <div id="home" class="pt-4 grid grid-rows-[auto_1fr]" >
                <Navbar/>
                <main    class=" scroll-mt-[10rem] grid grid-rows-[1fr] pt-4 gap-6       ">
                    <Routes>
                        <Route path="" view=HomePage/>
                        <Route path="/gallery" view=GalleryPage/>
                        <Route path="/*any" view=NotFound/>
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
