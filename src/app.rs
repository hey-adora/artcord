use chrono::Utc;
use leptos::logging::log;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::core::ConnectionReadyState;
use leptos_use::{use_websocket, UseWebsocketReturn};
use rkyv::Deserialize;

use components::navbar::Navbar;
use pages::gallery::GalleryPage;
use pages::home::HomePage;
use pages::not_found::NotFound;

use crate::app::utils::GlobalState;
use crate::server::{ClientMsg, ServerMsg};

mod components;
mod pages;
mod utils;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(GlobalState::new());
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");

    let (connected, set_connected) = create_signal(String::new());

    if cfg!(feature = "hydrate") {
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

        create_effect(move |_| {
            log!("{:?}", message.get());
        });

        create_effect(move |_| {
            let Some(bytes) = message_bytes.get() else {
                log!("Empty byte msg received.");
                return;
            };

            let server_msg = ServerMsg::from_bytes(&bytes);
            let Ok(server_msg) = server_msg else {
                log!("Error decoding msg: {}", server_msg.err().unwrap());
                return;
            };

            match server_msg {
                ServerMsg::Imgs(imgs) => {
                    log!("MSG RECEIVED: {:#?}", imgs);
                }
                ServerMsg::Reset => {
                    log!("RESETING");
                }
            };
        });

        create_effect(move |_| {
            set_connected(format!("{}", ready_state.get()));
        });

        create_effect(move |_| {
            if ready_state.get() == ConnectionReadyState::Open {
                send("test69");

                let msg = ClientMsg::GalleryInit {
                    amount: 25,
                    from: Utc::now().timestamp_nanos(),
                };
                let bytes = rkyv::to_bytes::<ClientMsg, 256>(&msg).unwrap();
                let bytes = bytes.into_vec();
                log!("{:?}", &bytes);
                send_bytes(bytes);
            }
        });
    };

    view! {
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        <Title text="Welcome to Leptos"/>
        <Body  class=move || format!("text-low-purple  bg-gradient-to-br from-mid-purple to-dark-purple   {}", if global_state.nav_open.get() == true { "overflow-hidden w-screen h-screen" } else { "" })  />
        <Router>
            <div id="home" class="pt-4 grid grid-rows-[auto_1fr]" >
                {move || connected()}
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
