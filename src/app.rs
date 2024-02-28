//use crate::app::components::gallery::SocketBs;
use crate::app::global_state::AuthState;
use crate::app::pages::admin::Admin;
use crate::app::pages::login::Login;
use crate::app::pages::register::{AuthLoadingState, Register};
use crate::app::utils::LoadingNotFound;
use crate::app::utils::ServerMsgImgResized;
use crate::app::utils::{resize_imgs, NEW_IMG_HEIGHT};
use crate::message::server_msg::ServerMsg;
use cfg_if::cfg_if;
use global_state::GlobalState;
use gloo_net::http::Request;
use leptos::logging::log;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::core::ConnectionReadyState;
use leptos_use::utils::Pausable;
use leptos_use::{
    use_interval_fn, use_websocket_with_options, use_window, UseWebSocketOptions,
    UseWebsocketReturn,
};
use pages::account::Account;
use pages::gallery::GalleryPage;
use pages::home::HomePage;
use pages::not_found::NotFound;
use pages::profile::Profile;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
//use web_sys::features::gen_WebSocket::WebSocket;

pub mod components;
pub mod global_state;
pub mod pages;
pub mod utils;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

enum WsConnectionState {
    Connecting,
    Connected,
    Disconnected,
}

pub fn get_ws_path() -> String {
    let default = String::from("wss://artcord.uk.to:3420");
    let mut output = String::new();
    let window = &*use_window();
    let Some(window) = window else {
        log!("Failed to get window for get_ws_path, using default ws path: {}", default);
        return default;
    };
    //let location = use_location();
    let protocol = window.location().protocol();
    let Ok(protocol) = protocol else {
        log!("Failed to get window for protocol, using default ws path: {}", default);
        return default;
    };
    if protocol == "http:" {
        output.push_str("ws://");
    } else {
        output.push_str("wss://");
    }
    let hostname = window.location().hostname();
    let Ok(hostname) = hostname else {
        log!("Failed to get window for hostname, using default ws path: {}", default);
        return default;
    };
    output.push_str(&format!("{}:3420", hostname));

    output
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(GlobalState::new());
    //provide_context(SocketBs::new());
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
  
    // let ws_state_chaneg_closures: RwSignal<Vec<Rc<dyn Fn(bool) -> ()>>> = RwSignal::new(Vec::new());
    //
  
    
    //WebSocket::OP
    //let ws_state: RwSignal<>
    //let (_connected, _set_connected) = create_signal(String::new());
    
    create_effect(move |_| {
        let ws_on_msg = global_state.ws_on_msg;
        let ws_on_err = global_state.ws_on_err;
        let ws_on_open = global_state.ws_on_open;
        let ws_on_close = global_state.ws_on_close;
        let ws = global_state.ws;
        
        log!("ONCE HOPEFULLY");
        ws_on_msg.set_value(Some(Rc::new(Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            let data = e.data().dyn_into::<js_sys::ArrayBuffer>();
            let Ok(data) = data else {
                return;
            };
            let array = js_sys::Uint8Array::new(&data);
            let bytes = array.to_vec();
            //log!("ONG MSG {:?}", vec);
            if bytes.is_empty() {
                log!("Empty byte msg received.");
                return;
            };

            let server_msg = ServerMsg::from_bytes(&bytes);
            let Ok((id, server_msg)) = server_msg else {
                log!("Error decoding msg: {}", server_msg.err().unwrap());
                return;
            };

            //log!("{:#?}", &server_msg);

            if id != 0 {
                global_state.execute(id, server_msg);
            } else {
                log!("IDDDDDDDD 0");
            }
        }))));
        ws_on_err.set_value(Some(Rc::new(Closure::<dyn FnMut(_)>::new(
            move |e: ErrorEvent| {
                log!("WS ERROR: {:?}", e);
            },
        ))));
        ws_on_open.set_value(Some(Rc::new(Closure::<dyn FnMut()>::new(move || {
            // ws_state_chaneg_closures.with_untracked(|closures| {
            //     for closure in closures {
            //         closure(true);
            //     }
            // });
            log!("CONNECTED");
        }))));
        ws_on_close.set_value(Some(Rc::new(Closure::<dyn FnMut()>::new(move || {
            // ws_state_chaneg_closures.with_untracked(|closures| {
            //     for closure in closures {
            //         closure(false);
            //     }
            // });
            log!("DISCONNECTED");
        }))));

        let create_ws = move || -> WebSocket {
            log!("CONNECTING");
            let ws = WebSocket::new("ws://localhost:3420").unwrap();
            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        
            ws_on_msg.with_value(|ws_on_msg| {
                if let Some(ws_on_msg) = ws_on_msg {
                    ws.set_onmessage(Some((**ws_on_msg).as_ref().unchecked_ref()));
                }
            });
    
            ws_on_err.with_value(|ws_on_err| {
                if let Some(ws_on_err) = ws_on_err {
                    ws.set_onerror(Some((**ws_on_err).as_ref().unchecked_ref()));
                }
            });
    
            ws_on_open.with_value(|ws_on_open| {
                if let Some(ws_on_open) = ws_on_open {
                    ws.set_onopen(Some((**ws_on_open).as_ref().unchecked_ref()));
                }
            });
    
            ws_on_close.with_value(|ws_on_close| {
                if let Some(ws_on_close) = ws_on_close {
                    ws.set_onclose(Some((**ws_on_close).as_ref().unchecked_ref()));
                }
            });
    
            // ws.set_onmessage(Some((*ws_on_msg.get_untracked()).as_ref().unchecked_ref()));
            // ws.set_onerror(Some((*ws_on_err.get_untracked()).as_ref().unchecked_ref()));
            // ws.set_onopen(Some((*ws_on_open.get_untracked()).as_ref().unchecked_ref()));
            // ws.set_onclose(Some(
            //     (*ws_on_close.get_untracked()).as_ref().unchecked_ref(),
            // ));

        
            ws
        };
        
        log!("AUTH_STATE: {:?}", global_state.auth_is_logged_out());
        // (reconnect_interval.resume)();
    
        ws.set_value(Some(create_ws()));
        let reconnect_interval = use_interval_fn(
            move || {
                let is_closed = ws.with_value(move |ws| {
                    ws.as_ref()
                        .and_then(|ws| Some(ws.ready_state() == WebSocket::CLOSED))
                        .unwrap_or(false)
                });
                if is_closed {
                    log!("RECONNECTING");
                    //ws.with_untracked(|ws| {});
                    ws.set_value(Some(create_ws()));
                }
            },
            1000,
        );
    });
    

    // cfg_if! {
    //     if #[cfg(target_arch = "wasm32")] {
    //         let ws = WebSocket::new("ws://localhost:3000/");
    //         ws.se
    //         log!("yo yo yo");
    //     } else if #[cfg(feature = "ssr")] {
    //         log!("no no no");
    //     }
    // }

    // if cfg!(feature = "hydrate") {
    //     //spawn_local(fut)
      

        
    //     let UseWebsocketReturn {
    //         ready_state,
    //         message,
    //         message_bytes,
    //         send_bytes,
    //         open,
    //         ..
    //     } = use_websocket_with_options(
    //         &get_ws_path(),
    //         UseWebSocketOptions::default()
    //             .on_open(move |e| {
    //                 global_state.socket_connected.set(true);
    //             })
    //             .on_close(move |e| {
    //                 global_state.socket_connected.set(false);
    //             })
    //             .on_message(move |msg| {
    //                 //log!("RECEIVED SOMETHING2: {:?}", &msg);
    //             })
    //             .on_message_bytes(move |bytes| {
    //                 //log!("RECEIVED SOMETHING: {:?}", &bytes);
    //                 if bytes.is_empty() {
    //                     log!("Empty byte msg received.");
    //                     return;
    //                 };

    //                 let server_msg = ServerMsg::from_bytes(&bytes);
    //                 let Ok((id, server_msg)) = server_msg else {
    //                     log!("Error decoding msg: {}", server_msg.err().unwrap());
    //                     return;
    //                 };

    //                 //log!("{:#?}", &server_msg);

    //                 if id != 0 {
    //                     global_state.execute(id, server_msg);
    //                 } else {
    //                     log!("IDDDDDDDD 0");
    //                 }

    //                 //let server_msg_name = server_msg.name();

    //                 // match server_msg {
    //                 //     ServerMsg::Reset => {
    //                 //         log!("RESETING");
    //                 //         //document().location().unwrap().replace("google.com")
    //                 //         //document().location().unwrap().reload().unwrap();
    //                 //     }
    //                 //     ServerMsg::None => {}
    //                 //     ServerMsg::Imgs(new_imgs) => {
    //                 //         if new_imgs.is_empty()
    //                 //             && global_state.page_galley.gallery_loaded.get_untracked()
    //                 //                 == LoadingNotFound::Loading
    //                 //         {
    //                 //             global_state
    //                 //                 .page_galley
    //                 //                 .gallery_loaded
    //                 //                 .set(LoadingNotFound::NotFound);
    //                 //         } else {
    //                 //             let new_imgs = new_imgs
    //                 //                 .iter()
    //                 //                 .map(|img| ServerMsgImgResized::from(img.to_owned()))
    //                 //                 .collect::<Vec<ServerMsgImgResized>>();

    //                 //             // if !global_state.page_galley.gallery_loaded.get_untracked() {
    //                 //             //     global_state.page_galley.gallery_loaded.set(true);
    //                 //             // }

    //                 //             global_state.page_galley.gallery_imgs.update(|imgs| {
    //                 //                 imgs.extend(new_imgs);
    //                 //                 let document = document();
    //                 //                 let gallery_section =
    //                 //                     document.get_element_by_id("gallery_section");
    //                 //                 let Some(gallery_section) = gallery_section else {
    //                 //                     return;
    //                 //                 };
    //                 //                 let width = gallery_section.client_width() as u32;
    //                 //                 resize_imgs(NEW_IMG_HEIGHT, width, imgs);
    //                 //             });
    //                 //             global_state
    //                 //                 .page_galley
    //                 //                 .gallery_loaded
    //                 //                 .set(LoadingNotFound::Loaded);
    //                 //         }
    //                 //     }
    //                 //     ServerMsg::ProfileImgs(new_imgs) => {
    //                 //         let Some(new_imgs) = new_imgs else {
    //                 //             //global_state.page_profile.gallery_loaded.set(true);
    //                 //             global_state
    //                 //                 .page_profile
    //                 //                 .gallery_loaded
    //                 //                 .set(LoadingNotFound::NotFound);
    //                 //             //global_state.socket_state_reset(&server_msg_name);
    //                 //             return;
    //                 //         };

    //                 //         if new_imgs.is_empty()
    //                 //             && global_state.page_profile.gallery_loaded.get_untracked()
    //                 //                 == LoadingNotFound::Loading
    //                 //         {
    //                 //             global_state
    //                 //                 .page_profile
    //                 //                 .gallery_loaded
    //                 //                 .set(LoadingNotFound::NotFound);
    //                 //         } else {
    //                 //             log!("PROFILE IMGS RECEIVED: {:?}", new_imgs.len());

    //                 //             let new_imgs = new_imgs
    //                 //                 .iter()
    //                 //                 .map(|img| ServerMsgImgResized::from(img.to_owned()))
    //                 //                 .collect::<Vec<ServerMsgImgResized>>();

    //                 //             global_state.page_profile.gallery_imgs.update(|imgs| {
    //                 //                 imgs.extend(new_imgs);
    //                 //                 let document = document();
    //                 //                 let gallery_section =
    //                 //                     document.get_element_by_id("profile_gallery_section");
    //                 //                 let Some(gallery_section) = gallery_section else {
    //                 //                     return;
    //                 //                 };
    //                 //                 let width = gallery_section.client_width() as u32;
    //                 //                 resize_imgs(NEW_IMG_HEIGHT, width, imgs);
    //                 //             });
    //                 //             global_state
    //                 //                 .page_profile
    //                 //                 .gallery_loaded
    //                 //                 .set(LoadingNotFound::Loaded);

    //                 //             // if !global_state.page_profile.gallery_loaded.get_untracked() {
    //                 //             //     global_state.page_profile.gallery_loaded.set(true);
    //                 //             // }
    //                 //         }
    //                 //     }
    //                 //     ServerMsg::Profile(new_user) => {
    //                 //         if let Some(new_user) = new_user {
    //                 //             //log!("USER RECEIVED: {:?}", &new_user.id);
    //                 //             global_state.page_profile.gallery_imgs.set(vec![]);
    //                 //             global_state
    //                 //                 .page_profile
    //                 //                 .gallery_loaded
    //                 //                 .set(LoadingNotFound::NotLoaded);
    //                 //             global_state.page_profile.user.update(move |user| {
    //                 //                 *user = Some(new_user);
    //                 //             });
    //                 //         } else {
    //                 //             global_state.page_profile.gallery_imgs.set(vec![]);
    //                 //             global_state
    //                 //                 .page_profile
    //                 //                 .gallery_loaded
    //                 //                 .set(LoadingNotFound::NotFound);
    //                 //             // log!("where is it????");
    //                 //         }
    //                 //     }
    //                 //     ServerMsg::RegistrationCompleted => {
    //                 //         global_state
    //                 //             .pages
    //                 //             .registration
    //                 //             .loading_state
    //                 //             .set(AuthLoadingState::Completed);
    //                 //     }
    //                 //     ServerMsg::RegistrationInvalid(invalid) => {
    //                 //         global_state
    //                 //             .pages
    //                 //             .registration
    //                 //             .loading_state
    //                 //             .set(AuthLoadingState::Failed(invalid));
    //                 //     }
    //                 //     ServerMsg::LoginInvalid(invalid) => {}
    //                 //     ServerMsg::LoginComplete { user_id, token } => {
    //                 //         //log!("TOKEN: {}", token);

    //                 //         let res = create_local_resource(
    //                 //             || {},
    //                 //             move |_| {
    //                 //                 let token = token.clone();
    //                 //                 async move {
    //                 //                     let resp = Request::post("/login_token").body(token);

    //                 //                     let Ok(resp) = resp else {
    //                 //                         log!("Login build error: {}", resp.err().unwrap());
    //                 //                         return;
    //                 //                     };

    //                 //                     let resp = resp.send().await;
    //                 //                     let Ok(resp) = resp else {
    //                 //                         log!("Login response error: {}", resp.err().unwrap());
    //                 //                         return;
    //                 //                     };

    //                 //                     log!("{:#?}", resp);
    //                 //                 }
    //                 //             },
    //                 //         );

    //                 //         global_state.auth.set(AuthState::LoggedIn { user_id });
    //                 //         global_state
    //                 //             .pages
    //                 //             .login
    //                 //             .loading_state
    //                 //             .set(AuthLoadingState::Completed);
    //                 //     }
    //                 //     ServerMsg::LoginFromTokenComplete { user_id } => {
    //                 //         global_state.auth.set(AuthState::LoggedIn { user_id });
    //                 //         global_state
    //                 //             .pages
    //                 //             .login
    //                 //             .loading_state
    //                 //             .set(AuthLoadingState::Completed);
    //                 //     }
    //                 //     ServerMsg::LoggedOut => {
    //                 //         log!("LOGGEDOUT");
    //                 //     }
    //                 //     ServerMsg::Ping => {
    //                 //         log!("PING");
    //                 //     }
    //                 // };
    //                 // global_state.socket_state_reset(&server_msg_name);
    //             })
    //             .immediate(true)
    //             .reconnect_limit(0)
    //             .reconnect_interval(10000),
    //     );
    //     global_state.socket_send_fn.set(Rc::new(send_bytes.clone()));

    //     let Pausable { pause, resume, .. } = use_interval_fn(
    //         move || {
    //             let state = ready_state.get_untracked();
    //             if state == ConnectionReadyState::Closed {
    //                 log!("RECONNECTING");
    //                 open();
    //             } else {
    //                 log!("{}", state);
    //             }
    //         },
    //         3000,
    //     );

    //     // create_effect(move |_| {
    //     //     log!("{:?}", message.get());
    //     // });

    //     // create_effect(move |_| {
    //     //     let Some(bytes) = message_bytes.get() else {
    //     //         log!("Empty byte msg received.");
    //     //         return;
    //     //     };
    //     //
    //     //     let server_msg = ServerMsg::from_bytes(&bytes);
    //     //     let Ok(server_msg) = server_msg else {
    //     //         log!("Error decoding msg: {}", server_msg.err().unwrap());
    //     //         return;
    //     //     };
    //     //
    //     //     match server_msg {
    //     //         ServerMsg::Reset => {
    //     //             log!("RESETING");
    //     //             document().location().unwrap().reload().unwrap();
    //     //         }
    //     //         msg => global_state.socket_recv.set(msg),
    //     //     };
    //     // });

    //     create_effect(move |_| {
    //         let state = ready_state.get();
    //         log!("SOCKET STATE: {}", state);
    //         match state {
    //             leptos_use::core::ConnectionReadyState::Closed => {
    //                 // let current_state = global_state.socket_connected.get_untracked();
    //                 // if current_state == true {
    //                 //     global_state.socket_connected.set(false);
    //                 // }
    //                 resume()
    //             }
    //             leptos_use::core::ConnectionReadyState::Open => {
    //                 // let current_state = global_state.socket_connected.get_untracked();
    //                 // if current_state == false {
    //                 //     global_state.socket_connected.set(true);
    //                 // }
    //                 //global_state.socket_connected.set(true);
    //                 pause()
    //             }
    //             _ => (),
    //         };
    //     });
    // };

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
                    <Route path="/user/:id" view=Profile/>
                    <Route path="/account" view=Account/>
                    <Route path="/admin" view=Admin/>
                    <Route path="/*any" view=NotFound/>
                    <ProtectedRoute condition=move || !global_state.auth_is_logged_out() redirect_path="/" path="/login" view=Login/>
                    <ProtectedRoute condition=move || !global_state.auth_is_logged_out() redirect_path="/"  path="/register" view=Register/>
                </Routes>
        </Router>
    }
}
