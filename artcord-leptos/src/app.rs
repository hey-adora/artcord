use crate::app::pages::admin::Admin;
use crate::app::pages::login::Login;
use crate::app::pages::register::Register;
use artcord_leptos_web_sockets::WsRuntime;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::debug_client_msg::DebugClientMsg;
use artcord_state::message::debug_msg_key::DebugMsgPermKey;
use artcord_state::message::debug_server_msg::DebugServerMsg;
use artcord_state::message::prod_server_msg::ServerMsg;
use global_state::GlobalState;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;


//use crate::app::utils::ws_runtime::WsRuntime;
//use artcord_leptos_web_sockets::Runtime;
use pages::account::Account;
use pages::main_gallery::MainGalleryPage;
use pages::home::HomePage;
use pages::not_found::NotFound;
use pages::user_gallery::UserGalleryPage;
use tracing::{trace, error};
use tracing::debug;
use cfg_if::cfg_if;

pub mod components;
pub mod global_state;
pub mod pages;
pub mod utils;

   




     

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(GlobalState::new());

   
    
    //let location = use_location();
    #[cfg(feature = "development")]
    {
        let debug_ws = WsRuntime::<u128, DebugMsgPermKey, DebugServerMsg, DebugClientMsg>::new();
        debug_ws.connect(3001).unwrap();
        // let ready_channel = debug_ws.create_singleton();
        
        //let client_msg = DebugClientMsg::BrowserReady;
        // ready_channel.send_once(client_msg, |server_msg| {
        //     trace!("server msg received: {:#?}", server_msg);
        // }).expect("failed to send");

        debug_ws.on_ws_state(move |is_connected| {
            if is_connected {
                trace!("ws_debug: sending browser ready package");
                match debug_ws.send(DebugMsgPermKey::Debug, DebugClientMsg::BrowserReady) {
                    Ok(result) => {
                        trace!("ws_debug: returned: {:?}", result);
                    }
                    Err(err) => {
                        error!("ws_debug: send error: {}", err);
                    }
                };
            }
        });
        
        debug_ws.on(DebugMsgPermKey::Debug, |server_msg| {
            trace!("ws_debug: Restart received: {:?}", server_msg);
            window().location().reload().unwrap();
        });
    }
   
    
   
    // debug_ws.on(DebugMsgPermKey::Restart, |server_msg| {
    //     debug!("Restart received 22222222222: {:?}", server_msg);
    // });
    // WsRuntime::connect("ws://localhost", "3001");
    // a a a a a a
    
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    global_state.ws.connect(3420).unwrap();

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
                    <Route path="/gallery" view=MainGalleryPage/>
                    <Route path="/user/:id" view=UserGalleryPage/>
                    <Route path="/account" view=Account/>
                    <Route path="/admin" view=Admin/>
                    <Route path="/*any" view=NotFound/>
                    <ProtectedRoute condition=move || !global_state.auth_is_logged_out() redirect_path="/" path="/login" view=Login/>
                    <ProtectedRoute condition=move || !global_state.auth_is_logged_out() redirect_path="/"  path="/register" view=Register/>
                </Routes>
        </Router>
    }
}
