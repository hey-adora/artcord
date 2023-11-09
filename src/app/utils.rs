use bson::DateTime;
use chrono::Utc;
use leptos::logging::log;
use leptos::*;
use leptos::{
    create_rw_signal, logging, window, RwSignal, SignalGet, SignalGetUntracked,
    SignalUpdateUntracked, SignalWithUntracked,
};
use std::sync::{Mutex, RwLock};
use std::{
    collections::{HashMap, LinkedList},
    rc::Rc,
};
use wasm_bindgen::JsValue;
use web_sys::Location;

use crate::{
    database::User,
    server::{ClientMsg, ServerMsgImg, SERVER_MSG_IMGS_NAME},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScrollSection {
    Home,
    About,
    Gallery,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ServerMsgImgResized {
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
            user: value.user,
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
    pub socket_state: RwSignal<Rc<Mutex<HashMap<String, i64>>>>,
}

impl GlobalState {
    pub fn new() -> Self {
        // let a = Utc::now();
        // let b = chrono::Duration::;
        // let c = chrono::DateTime::timestamp_nanos_opt(&self);
        Self {
            section: create_rw_signal(ScrollSection::Home),
            nav_open: create_rw_signal(false),
            nav_tran: create_rw_signal(true),
            socket_send: create_rw_signal(Rc::new(|_| {})),
            gallery_imgs: create_rw_signal(Vec::new()),
            socket_state: create_rw_signal(Rc::new(Mutex::new(HashMap::new()))),
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

    pub fn socket_state_imgs_is_ready(&self) -> bool {
        // let socket_state =
        //     self.socket_state.with(
        //         |state| match state.get(&String::from(SERVER_MSG_IMGS_NAME)) {
        //             Some(n) => Some(*n),
        //             None => None,
        //         },
        //     );

        let socket_state = self.socket_state.with(|state| {
            match state
                .lock()
                .unwrap()
                .get(&String::from(SERVER_MSG_IMGS_NAME))
            {
                Some(n) => Some(*n),
                None => None,
            }
        });

        let Some(n) = socket_state else {
            // log!("YO READY");
            return true;
        };
        let now = Utc::now().timestamp_nanos();
        let diff = now - n;
        let is_ready = diff >= 2_000_000_000;
        // log!("IS IT READY?: {} - {} >= {} {}", now, n, diff, is_ready);
        is_ready
    }

    pub fn socket_state_imgs_reset(&self) {
        // log!("REMOVED?? WHY???");
        self.socket_state.update(|state| {
            state
                .lock()
                .unwrap()
                .remove(&String::from(SERVER_MSG_IMGS_NAME));
        });
    }

    pub fn socket_state_imgs_used(&self) {
        // log!("USED");
        self.socket_state.update(|state| {
            let name = String::from(SERVER_MSG_IMGS_NAME);
            let mut locked_state = state.lock().unwrap();
            let Some(state) = locked_state.get_mut(&name) else {
                locked_state.insert(name, Utc::now().timestamp_nanos());
                return;
            };

            *state = Utc::now().timestamp_nanos();
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
