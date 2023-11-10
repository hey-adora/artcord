use self::utils::ServerMsgImgResized;
use crate::app::utils::GlobalState;
use crate::server::ServerMsg;
use components::navbar::Navbar;
use leptos::logging::log;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::{use_websocket, UseWebsocketReturn};
use pages::gallery::GalleryPage;
use pages::home::HomePage;
use pages::not_found::NotFound;
use std::rc::Rc;

mod components;
mod pages;
mod utils;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(GlobalState::new());
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let (_connected, _set_connected) = create_signal(String::new());

    if cfg!(feature = "hydrate") {
        let UseWebsocketReturn {
            ready_state,
            message,
            message_bytes,
            send_bytes,
            ..
        } = use_websocket("/ws/");
        global_state.socket_send.set(Rc::new(send_bytes.clone()));

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

            let name = server_msg.name();

            match server_msg {
                ServerMsg::Imgs(new_imgs) => {
                    global_state.gallery_imgs.update(|imgs| {
                        imgs.extend_from_slice(
                            &new_imgs
                                .into_iter()
                                .map(|img| ServerMsgImgResized::from(img))
                                .collect::<Vec<ServerMsgImgResized>>(),
                        );
                    });
                }
                ServerMsg::Reset => {
                    log!("RESETING");
                    document().location().unwrap().reload().unwrap();
                }
            };

            global_state.socket_state_imgs_reset(&name);
        });

        create_effect(move |_| {
            log!("SOCKET STATE: {}", ready_state.get());
        });
    };

    view! {
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        <Title text="Welcome to Leptos"/>
        <Body  class=move || format!("text-low-purple  bg-gradient-to-br from-mid-purple to-dark-purple   {}", if global_state.nav_open.get() == true { "overflow-hidden w-screen h-screen" } else { "" })  />
        <Router>
            <div id="home" class="pt-4 grid grid-rows-[auto_1fr] min-h-screen" >
                // {move || connected()}
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
