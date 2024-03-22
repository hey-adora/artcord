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

use super::img_resize::GalleryImg;

#[derive(Clone, PartialEq, Debug)]
pub struct ServerMsgImgResized {
    pub id: String,
    // pub id: u128,
    pub quality: ImgQuality,
    pub display_high: String,
    pub display_preview: String,
    pub user: User,
    pub user_id: String,
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
    pub modified_at: i64,
    pub created_at: i64,
}

// Hi

impl Default for ServerMsgImgResized {
    fn default() -> Self {
        Self {
            id: String::from("1177244237021073450"),
            quality: ImgQuality::Org,
            display_preview: String::from(
                "/assets/gallery/org_2552bd2db66978a9b3675721e95d1cbd.png",
            ),
            display_high: String::from("/assets/gallery/org_2552bd2db66978a9b3675721e95d1cbd.png"),
            user: User {
                id: String::from("id"),
                guild_id: String::from("1159766826620817419"),
                name: String::from("name"),
                pfp_hash: Some(String::from("pfp_hash")),
                modified_at: Utc::now().timestamp_millis(),
                created_at: Utc::now().timestamp_millis(),
            },
            user_id: String::from("1159037321283375174"),
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
            modified_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
        }
    }
}

impl GalleryImg for ServerMsgImgResized {
 
    fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32) {
        self.left.set(left);
        self.top.set(top);
        self.new_width.set(new_width);
        self.new_height.set(new_height);
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn get_pos(&self) -> (f32, f32) {
        (self.left.get_untracked(), self.top.get_untracked())
    }
}

impl From<AggImg> for ServerMsgImgResized {
    fn from(value: AggImg) -> Self {
        let quality = value.pick_quality();
        let display_preview = quality.gen_link_preview(&value.org_hash, &value.format);
        Self {
            id: value.id,
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

pub fn create_client_test_imgs() -> Vec<ServerMsgImgResized> {
    let mut new_imgs: Vec<ServerMsgImgResized> = Vec::new();
    for _ in 0..25 {
        new_imgs.push(ServerMsgImgResized::default());
    }
    new_imgs
}
