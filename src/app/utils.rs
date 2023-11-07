use std::{collections::LinkedList, rc::Rc};

use leptos::{create_rw_signal, window, RwSignal, SignalGet, SignalGetUntracked};
use wasm_bindgen::JsValue;
use web_sys::Location;

use crate::server::{ClientMsg, ServerMsgImg};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScrollSection {
    Home,
    About,
    Gallery,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ServerMsgImgResized {
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
    pub modified_at: i64,
    pub created_at: i64,
}

impl From<ServerMsgImg> for ServerMsgImgResized {
    fn from(value: ServerMsgImg) -> Self {
        Self {
            new_width: value.width,
            new_height: value.height,
            user_id: value.user_id,
            msg_id: value.msg_id,
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
    pub socket_send: RwSignal<Rc<dyn Fn(Vec<u8>)>>,
    pub gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            section: create_rw_signal(ScrollSection::Home),
            nav_open: create_rw_signal(false),
            nav_tran: create_rw_signal(true),
            socket_send: create_rw_signal(Rc::new(|_| {})),
            gallery_imgs: create_rw_signal(Vec::new()),
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
        // leptos::logging::log!("{:?}", &bytes);
        self.socket_send.get_untracked()(bytes);
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
