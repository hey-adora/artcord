use artcord_state::aggregation::server_msg_img::AggImg;
use artcord_state::misc::img_quality::ImgQuality;
use artcord_state::model::user::User;
use chrono::Utc;
use leptos::*;
use leptos::{window, RwSignal, SignalGetUntracked};
use std::fmt::Debug;
use wasm_bindgen::JsValue;
use web_sys::Location;
use tracing::{trace, debug};

use self::img_resize::GalleryImg;

pub mod img_resize;
pub mod img_resized;
pub mod signal_switch;


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LoadingNotFound {
    NotLoaded,
    Loading,
    Loaded,
    NotFound,
    Error,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScrollSection {
    Home,
    About,
    Gallery,
    UserProfile,
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

#[derive(Clone)]
pub struct SelectedImg {
    pub org_url: String,
    pub author_name: String,
    pub author_pfp: String,
    pub author_id: String,
    pub width: u32,
    pub height: u32,
}
