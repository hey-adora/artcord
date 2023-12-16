use bson::oid::ObjectId;
use bson::DateTime;
use chrono::Utc;

use leptos::*;
use leptos::{create_rw_signal, window, RwSignal, SignalGetUntracked};
use leptos_use::core::ConnectionReadyState;
use rand::Rng;
use std::{collections::HashMap, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::Location;

use crate::bot::ImgQuality;
use crate::server::ServerMsg;
use crate::{
    database::User,
    server::{ClientMsg, ServerMsgImg},
};

use super::components::gallery::GalleryImg;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScrollSection {
    Home,
    About,
    Gallery,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ServerMsgImgResized {
    pub _id: ObjectId,
    // pub id: u128,
    pub quality: ImgQuality,
    pub display_high: String,
    pub display_preview: String,
    pub user: User,
    pub user_id: String,
    pub msg_id: String,
    pub org_hash: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub new_width: RwSignal<f32>,
    pub new_height: RwSignal<f32>,
    pub top: RwSignal<f32>,
    pub left: RwSignal<f32>,
    pub has_high: bool,
    pub has_medium: bool,
    pub has_low: bool,
    pub modified_at: DateTime,
    pub created_at: DateTime,
}

impl Default for ServerMsgImgResized {
    fn default() -> Self {
        Self {
            _id: ObjectId::new(),
            quality: ImgQuality::Org,
            display_preview: String::from(
                "/assets/gallery/org_2552bd2db66978a9b3675721e95d1cbd.png",
            ),
            display_high: String::from("/assets/gallery/org_2552bd2db66978a9b3675721e95d1cbd.png"),
            user: User {
                _id: ObjectId::new(),
                guild_id: String::from("1159766826620817419"),
                id: String::from("id"),
                name: String::from("name"),
                pfp_hash: Some(String::from("pfp_hash")),
                modified_at: DateTime::from_millis(Utc::now().timestamp_millis()),
                created_at: DateTime::from_millis(Utc::now().timestamp_millis()),
            },
            user_id: String::from("1159037321283375174"),
            msg_id: String::from("1177244237021073450"),
            org_hash: String::from("2552bd2db66978a9b3675721e95d1cbd"),
            format: String::from("png"),
            width: 233,
            height: 161,
            new_width: RwSignal::new(233.0),
            new_height: RwSignal::new(161.0),
            top: RwSignal::new(0.0),
            left: RwSignal::new(0.0),
            has_high: false,
            has_medium: false,
            has_low: false,
            modified_at: DateTime::from_millis(Utc::now().timestamp_millis()),
            created_at: DateTime::from_millis(Utc::now().timestamp_millis()),
        }
    }
}

impl GalleryImg for ServerMsgImgResized {
    fn mark_as_modified(&mut self, id: u128) {
        // self.id = id;
    }
    fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32) {
        self.left.set(left);
        self.top.set(top);
        self.new_width.set(new_width);
        self.new_height.set(new_height);
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl From<ServerMsgImg> for ServerMsgImgResized {
    fn from(value: ServerMsgImg) -> Self {
        let quality = value.pick_quality();
        let display_preview = quality.gen_link_preview(&value.org_hash, &value.format);
        Self {
            _id: value._id,
            quality,
            display_preview,
            // id: rand::thread_rng().gen::<u128>(),
            display_high: ImgQuality::gen_link_org(&value.org_hash, &value.format),
            user: value.user,
            new_width: RwSignal::new(value.width as f32),
            new_height: RwSignal::new(value.height as f32),
            top: RwSignal::new(0.0),
            left: RwSignal::new(0.0),
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
