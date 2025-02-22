use crate::app::pages::register::GlobalAuthState;

use artcord_leptos_web_sockets::WsRuntime;
use artcord_state::global;
use leptos::{create_rw_signal, RwSignal, SignalWith, StoredValue};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
// use crate::app::utils::server_msg_wrap::ServerMsgWrap;

use super::{
    pages::{
        admin::AdminPageState, main_gallery::GalleryPageState, user_gallery::PageUserGalleryState,
    },
    utils::PageUrl,
};

#[derive(Clone, Copy)]
pub struct GlobalState {
    pub auth: RwSignal<AuthState>,
    pub current_page_url: RwSignal<PageUrl>,
    pub nav_open: RwSignal<bool>,
    pub nav_tran: RwSignal<bool>,
    pub page_profile: PageUserGalleryState,
    pub pages: Pages,
    // pub current_page_url: PageUrl,
    // pub socket_timestamps: RwSignal<HashMap<&'static str, i64>>,
    // pub socket_connected: RwSignal<bool>,
    // pub socket_closures: StoredValue<HashMap<u128, Rc<dyn Fn(ServerMsg)>>>,
    // pub socket_pending_client_msgs: StoredValue<Vec<u8>>,
    pub ws: WsRuntime<global::ServerMsg, global::ClientMsg>,
    //pub ws: u64,
}

#[derive(Clone, Debug)]
pub enum AuthState {
    Processing,
    LoggedIn { user_id: String },
    LoggedOut,
}

#[derive(Copy, Clone, Debug)]
pub struct Pages {
    pub registration: GlobalAuthState,
    pub login: GlobalAuthState,
    pub gallery: GalleryPageState,
    pub admin: AdminPageState,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            registration: GlobalAuthState::new(),
            login: GlobalAuthState::new(),
            gallery: GalleryPageState::new(),
            admin: AdminPageState::new(),
        }
    }
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            auth: create_rw_signal(AuthState::Processing),
            current_page_url: create_rw_signal(PageUrl::Home),
            nav_open: create_rw_signal(false),
            nav_tran: create_rw_signal(true),
            page_profile: PageUserGalleryState::new(),
            pages: Pages::new(),
            // socket_connected: create_rw_signal(false),
            // socket_timestamps: create_rw_signal(HashMap::new()),
            // socket_closures: StoredValue::new(HashMap::new()),
            // socket_pending_client_msgs: StoredValue::new(Vec::new()),
            ws: WsRuntime::<global::ServerMsg, global::ClientMsg>::new(),
            //ws: 0,
        }
    }

    pub fn auth_is_processing(&self) -> bool {
        self.auth.with(|a| match a {
            AuthState::Processing => true,
            _ => false,
        })
    }

    pub fn auth_is_logged_in(&self) -> bool {
        self.auth.with(|a| match a {
            AuthState::LoggedIn { user_id: _ } => true,
            _ => false,
        })
    }

    pub fn auth_is_logged_out(&self) -> bool {
        self.auth.with(|a| match a {
            AuthState::LoggedOut => true,
            _ => false,
        })
    }
}
