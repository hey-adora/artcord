use crate::app::pages::login::Login;
use crate::app::pages::register::{AuthLoadingState, Register};
use crate::app::utils::LoadingNotFound;
use crate::app::utils::ServerMsgImgResized;
use crate::app::utils::{resize_imgs, NEW_IMG_HEIGHT};
use crate::server::server_msg::ServerMsg;
use global_state::GlobalState;
use gloo_net::http::Request;
use leptos::logging::log;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::core::ConnectionReadyState;
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
pub mod global_state;
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
                        ServerMsg::None => {}
                        ServerMsg::Imgs(new_imgs) => {
                            if new_imgs.is_empty()
                                && global_state.page_galley.gallery_loaded.get_untracked()
                                    == LoadingNotFound::Loading
                            {
                                global_state
                                    .page_galley
                                    .gallery_loaded
                                    .set(LoadingNotFound::NotFound);
                            } else {
                                let new_imgs = new_imgs
                                    .iter()
                                    .map(|img| ServerMsgImgResized::from(img.to_owned()))
                                    .collect::<Vec<ServerMsgImgResized>>();

                                // if !global_state.page_galley.gallery_loaded.get_untracked() {
                                //     global_state.page_galley.gallery_loaded.set(true);
                                // }

                                global_state.page_galley.gallery_imgs.update(|imgs| {
                                    imgs.extend(new_imgs);
                                    let document = document();
                                    let gallery_section =
                                        document.get_element_by_id("gallery_section");
                                    let Some(gallery_section) = gallery_section else {
                                        return;
                                    };
                                    let width = gallery_section.client_width() as u32;
                                    resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                                });
                                global_state
                                    .page_galley
                                    .gallery_loaded
                                    .set(LoadingNotFound::Loaded);
                            }
                        }
                        ServerMsg::ProfileImgs(new_imgs) => {
                            let Some(new_imgs) = new_imgs else {
                                //global_state.page_profile.gallery_loaded.set(true);
                                global_state
                                    .page_profile
                                    .gallery_loaded
                                    .set(LoadingNotFound::NotFound);
                                //global_state.socket_state_reset(&server_msg_name);
                                return;
                            };

                            if new_imgs.is_empty()
                                && global_state.page_profile.gallery_loaded.get_untracked()
                                    == LoadingNotFound::Loading
                            {
                                global_state
                                    .page_profile
                                    .gallery_loaded
                                    .set(LoadingNotFound::NotFound);
                            } else {
                                //log!("PROFILE IMGS RECEIVED: {:?}", new_imgs.len());

                                let new_imgs = new_imgs
                                    .iter()
                                    .map(|img| ServerMsgImgResized::from(img.to_owned()))
                                    .collect::<Vec<ServerMsgImgResized>>();

                                global_state.page_profile.gallery_imgs.update(|imgs| {
                                    imgs.extend(new_imgs);
                                    let document = document();
                                    let gallery_section =
                                        document.get_element_by_id("profile_gallery_section");
                                    let Some(gallery_section) = gallery_section else {
                                        return;
                                    };
                                    let width = gallery_section.client_width() as u32;
                                    resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                                });
                                global_state
                                    .page_profile
                                    .gallery_loaded
                                    .set(LoadingNotFound::Loaded);

                                // if !global_state.page_profile.gallery_loaded.get_untracked() {
                                //     global_state.page_profile.gallery_loaded.set(true);
                                // }
                            }
                        }
                        ServerMsg::Profile(new_user) => {
                            if let Some(new_user) = new_user {
                                //log!("USER RECEIVED: {:?}", &new_user.id);
                                global_state.page_profile.gallery_imgs.set(vec![]);
                                global_state
                                    .page_profile
                                    .gallery_loaded
                                    .set(LoadingNotFound::NotLoaded);
                                global_state.page_profile.user.update(move |user| {
                                    *user = Some(new_user);
                                });
                            } else {
                                global_state.page_profile.gallery_imgs.set(vec![]);
                                global_state
                                    .page_profile
                                    .gallery_loaded
                                    .set(LoadingNotFound::NotFound);
                                // log!("where is it????");
                            }
                        }
                        ServerMsg::RegistrationCompleted => {
                            global_state
                                .pages
                                .registration
                                .loading_state
                                .set(AuthLoadingState::Completed);
                        }
                        ServerMsg::RegistrationInvalid(invalid) => {
                            global_state
                                .pages
                                .registration
                                .loading_state
                                .set(AuthLoadingState::Failed(invalid));
                        }
                        ServerMsg::LoginInvalid(invalid) => {}
                        ServerMsg::LoginComplete(token) => {
                            //log!("TOKEN: {}", token);

                            let res = create_local_resource(
                                || {},
                                move |_| {
                                    let token = token.clone();
                                    async move {
                                        let resp = Request::post("/login_token").body(token);

                                        let Ok(resp) = resp else {
                                            return;
                                        };

                                        let resp = resp.send().await;
                                        let Ok(resp) = resp else {
                                            return;
                                        };

                                        log!("{:#?}", resp);
                                    }
                                },
                            );
                        }
                    };
                    global_state.socket_state_reset(&server_msg_name);
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

        // create_effect(move |_| {
        //     log!("{:?}", message.get());
        // });

        // create_effect(move |_| {
        //     let Some(bytes) = message_bytes.get() else {
        //         log!("Empty byte msg received.");
        //         return;
        //     };
        //
        //     let server_msg = ServerMsg::from_bytes(&bytes);
        //     let Ok(server_msg) = server_msg else {
        //         log!("Error decoding msg: {}", server_msg.err().unwrap());
        //         return;
        //     };
        //
        //     match server_msg {
        //         ServerMsg::Reset => {
        //             log!("RESETING");
        //             document().location().unwrap().reload().unwrap();
        //         }
        //         msg => global_state.socket_recv.set(msg),
        //     };
        // });

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
                    //global_state.socket_connected.set(true);
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
        <meta name="twitter:title" content="ArtCord"/>
        <meta name="twitter:description" content="Art Community!"/>
        <meta name="twitter:image" content="/assets/overview.webp"/>
        <meta name="twitter:card" content="summary_large_image"/>
        <meta http-equiv="Cache-Control" content="no-cache, no-store, must-revalidate"/>
        <meta http-equiv="Pragma" content="no-cache"/>
        <meta http-equiv="Expires" content="0"/>

        <Stylesheet id="leptos" href="/pkg/leptos_start5.css"/>
        <Title text="ArtCord"/>
        <Body  class=move || format!("text-low-purple    bg-fixed bg-sword-lady  bg-[right_65%_bottom_0] md:bg-center bg-cover bg-no-repeat  bg-dark-night2 {}", if global_state.nav_open.get() == true { "overflow-hidden w-screen h-[dvh]" } else { "" })  />
        <Router>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/gallery" view=GalleryPage/>
                    <Route path="/login" view=Login/>
                    <Route path="/register" view=Register/>
                    <Route path="/user/:id" view=Profile/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
        </Router>
    }
}
