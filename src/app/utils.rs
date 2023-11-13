use bson::DateTime;
use chrono::Utc;

use leptos::*;
use leptos::{create_rw_signal, window, RwSignal, SignalGetUntracked};
use leptos_use::core::ConnectionReadyState;
use std::{collections::HashMap, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::Location;

use crate::server::ServerMsg;
use crate::{
    database::User,
    server::{ClientMsg, ServerMsgImg},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScrollSection {
    Home,
    About,
    Gallery,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ServerMsgImgResized {
    pub display_high: String,
    pub display_preview: String,
    pub user: User,
    pub user_id: String,
    pub msg_id: String,
    pub org_hash: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub new_width: u32,
    pub new_height: u32,
    pub has_high: bool,
    pub has_medium: bool,
    pub has_low: bool,
    pub modified_at: DateTime,
    pub created_at: DateTime,
}

impl From<ServerMsgImg> for ServerMsgImgResized {
    fn from(value: ServerMsgImg) -> Self {
        Self {
            display_high: String::new(),
            display_preview: String::new(),
            user: value.user,
            new_width: value.width,
            new_height: value.height,
            user_id: value.user_id,
            msg_id: value.id,
            org_hash: value.org_hash,
            format: value.format,
            width: value.width,
            height: value.height,
            has_high: value.has_high,
            has_medium: value.has_medium,
            has_low: value.has_low,
            modified_at: value.modified_at,
            created_at: value.created_at,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GlobalState {
    pub section: RwSignal<ScrollSection>,
    pub nav_open: RwSignal<bool>,
    pub nav_tran: RwSignal<bool>,
    pub socket_connected: RwSignal<bool>,
    pub socket_send: RwSignal<Rc<dyn Fn(Vec<u8>)>>,
    pub socket_recv: RwSignal<ServerMsg>,
    pub socket_timestamps: RwSignal<HashMap<&'static str, i64>>,
    pub gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
    pub gallery_loaded: RwSignal<bool>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            section: create_rw_signal(ScrollSection::Home),
            nav_open: create_rw_signal(false),
            nav_tran: create_rw_signal(true),
            socket_send: create_rw_signal(Rc::new(|_| {})),
            socket_connected: create_rw_signal(false),
            socket_recv: create_rw_signal(ServerMsg::None),
            socket_timestamps: create_rw_signal(HashMap::new()),
            gallery_imgs: create_rw_signal(Vec::new()),
            gallery_loaded: create_rw_signal(false),
        }
    }

    pub fn socket_send(&self, client_msg: ClientMsg) {
        let bytes = rkyv::to_bytes::<ClientMsg, 256>(&client_msg);
        let Ok(bytes) = bytes else {
            println!(
                "Failed to serialize client msg: {:?}, error: {}",
                &client_msg,
                bytes.err().unwrap()
            );
            return;
        };
        let bytes = bytes.into_vec();
        self.socket_send.get_untracked()(bytes);
    }

    pub fn socket_state_is_ready(&self, name: &str) -> bool {
        let socket_state = self
            .socket_timestamps
            .with_untracked(|state| match state.get(name) {
                Some(n) => Some(*n),
                None => None,
            });

        let Some(n) = socket_state else {
            return true;
        };
        let now = Utc::now().timestamp_nanos_opt().unwrap();
        let diff = now - n;
        let is_ready = diff >= 2_000_000_000;
        is_ready
    }

    pub fn socket_state_reset(&self, name: &str) {
        self.socket_timestamps.update_untracked(|state| {
            state.remove(name);
        });
    }

    pub fn socket_state_used(&self, name: &'static str) {
        self.socket_timestamps.update_untracked(move |state| {
            let Some(socket_state) = state.get_mut(name) else {
                state.insert(name, Utc::now().timestamp_nanos_opt().unwrap());
                return;
            };

            *socket_state = Utc::now().timestamp_nanos_opt().unwrap();
        });
    }
}

pub fn get_window_path() -> String {
    let location: Location = window().location();
    let path: Result<String, JsValue> = location.pathname();
    let hash: Result<String, JsValue> = location.hash();
    if let (Ok(path), Ok(hash)) = (path, hash) {
        format!("{}{}", path, hash)
    } else {
        String::from("/")
    }
}
