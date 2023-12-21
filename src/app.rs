use crate::app::components::gallery::{resize_imgs, NEW_IMG_HEIGHT};
use crate::app::utils::GlobalState;
use crate::app::utils::ServerMsgImgResized;
use crate::server::ServerMsg;
use leptos::logging::log;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::core::ConnectionReadyState;
use leptos_use::use_document;
use leptos_use::utils::Pausable;
use leptos_use::{
    use_interval_fn, use_websocket_with_options, UseWebSocketOptions, UseWebsocketReturn,
};
use pages::gallery::GalleryPage;
use pages::home::HomePage;
use pages::not_found::NotFound;
use pages::profile::Profile;
use std::rc::Rc;

pub mod components;
pub mod pages;
pub mod utils;

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
            open,
            ..
        } = use_websocket_with_options(
            "/ws/",
            UseWebSocketOptions::default()
                .on_message_bytes(move |bytes| {
                    if bytes.is_empty() {
                        log!("Empty byte msg received.");
                        return;
                    };

                    let server_msg = ServerMsg::from_bytes(&bytes);
                    let Ok(server_msg) = server_msg else {
                        log!("Error decoding msg: {}", server_msg.err().unwrap());
                        return;
                    };

                    let server_msg_name = server_msg.name();

                    match server_msg {
                        ServerMsg::Reset => {
                            log!("RESETING");
                            document().location().unwrap().reload().unwrap();
                        }
                        ServerMsg::Imgs(new_imgs) => {
                            let new_imgs = new_imgs
                                .iter()
                                .map(|img| ServerMsgImgResized::from(img.to_owned()))
                                .collect::<Vec<ServerMsgImgResized>>();

                            global_state.gallery_imgs.update(|imgs| {
                                imgs.extend(new_imgs);
                                let document = document();
                                let gallery_section = document.get_element_by_id("gallery_section");
                                let Some(gallery_section) = gallery_section else {
                                    return;
                                };
                                let width = gallery_section.client_width() as u32;
                                resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                            });

                            global_state.socket_state_reset(&server_msg_name);
                        }
                        msg => global_state.socket_recv.set(msg),
                    };
                })
                .immediate(true)
                .reconnect_limit(0)
                .reconnect_interval(10000),
        );
        global_state.socket_send.set(Rc::new(send_bytes.clone()));

        let Pausable { pause, resume, .. } = use_interval_fn(
            move || {
                let state = ready_state.get_untracked();
                if state == ConnectionReadyState::Closed {
                    log!("RECONNECTING");
                    open();
                } else {
                    log!("{}", state);
                }
            },
            3000,
        );

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
                ServerMsg::Reset => {
                    log!("RESETING");
                    document().location().unwrap().reload().unwrap();
                }
                msg => global_state.socket_recv.set(msg),
            };
        });

        create_effect(move |_| {
            let state = ready_state.get();
            log!("SOCKET STATE: {}", state);
            match state {
                leptos_use::core::ConnectionReadyState::Closed => {
                    let current_state = global_state.socket_connected.get_untracked();
                    if current_state == true {
                        global_state.socket_connected.set(false);
                    }
                    resume()
                }
                leptos_use::core::ConnectionReadyState::Open => {
                    let current_state = global_state.socket_connected.get_untracked();
                    if current_state == false {
                        global_state.socket_connected.set(true);
                    }
                    global_state.socket_connected.set(true);
                    pause()
                }
                _ => (),
            };
        });
    };

    view! {
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <meta name="description" content="Art Community!"/>
        <meta name="keywords" content="artcord,art,gallery,server,discord,community"/>
        <meta name="twitter:title" content="Artcord"/>
        <meta name="twitter:description" content="Art Community!"/>
        <meta name="twitter:image" content="/assets/overview.webp"/>
        <meta name="twitter:card" content="summary_large_image"/>
        <meta http-equiv="Cache-Control" content="no-cache, no-store, must-revalidate"/>
        <meta http-equiv="Pragma" content="no-cache"/>
        <meta http-equiv="Expires" content="0"/>

        <Stylesheet id="leptos" href="/pkg/leptos_start2.css"/>
        <Title text="Artcord"/>
        <Body  class=move || format!("text-low-purple    bg-fixed bg-sword-lady  bg-[right_65%_bottom_0] md:bg-center bg-cover bg-no-repeat  bg-dark-night2 {}", if global_state.nav_open.get() == true { "overflow-hidden w-screen h-[dvh]" } else { "" })  />
        <Router>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/gallery" view=GalleryPage/>
                    <Route path="/user/:id" view=Profile/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
        </Router>
    }
}
