use bson::oid::ObjectId;
use bson::DateTime;
use chrono::Utc;

use leptos::*;
use leptos::logging::log;
use leptos::{create_rw_signal, window, RwSignal, SignalGetUntracked};
use leptos_use::core::ConnectionReadyState;
use rand::Rng;
use std::{collections::HashMap, rc::Rc};
use std::fmt::Debug;
use wasm_bindgen::JsValue;
use web_sys::Location;

use crate::bot::ImgQuality;
use crate::server::ServerMsg;
use crate::{
    database::User,
    server::{ClientMsg, ServerMsgImg},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LoadingNotFound {
    NotLoaded,
    Loading,
    Loaded,
    NotFound
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScrollSection {
    Home,
    About,
    Gallery,
    UserProfile,
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
    fn get_pos(&self) -> (f32, f32) {
        (self.left.get_untracked(), self.top.get_untracked())
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
pub struct PageProfileState {
   // pub not_found: RwSignal<bool>,
    pub user: RwSignal<Option<User>>,
    pub gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
    pub gallery_loaded: RwSignal<LoadingNotFound>,
}

impl PageProfileState {
    pub fn new() -> Self {
        Self {
    //        not_found: RwSignal::new(false),
            user: RwSignal::new(None),
            gallery_imgs: RwSignal::new(Vec::new()),
            gallery_loaded: RwSignal::new(LoadingNotFound::NotLoaded),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PageGalleryState {
    pub gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
    pub gallery_loaded: RwSignal<LoadingNotFound>,
}

impl PageGalleryState {
    pub fn new() -> Self {
        Self {
            gallery_imgs: create_rw_signal(Vec::new()),
            gallery_loaded: create_rw_signal(LoadingNotFound::NotLoaded),
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
    pub page_galley: PageGalleryState,
    pub page_profile: PageProfileState,
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
            page_galley: PageGalleryState::new(),
            page_profile: PageProfileState::new(),
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

#[derive(Clone)]
pub struct SelectedImg {
    pub org_url: String,
    pub author_name: String,
    pub author_pfp: String,
    pub author_id: String,
    pub width: u32,
    pub height: u32,
}

pub const NEW_IMG_HEIGHT: u32 = 250;

pub trait GalleryImg {
    fn get_size(&self) -> (u32, u32);
    fn get_pos(&self) -> (f32, f32);
    fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32);
    fn mark_as_modified(&mut self, id: u128);
}

pub fn resize_img<T: GalleryImg + Debug>(
    top: &mut f32,
    max_width: u32,
    new_row_start: usize,
    new_row_end: usize,
    imgs: &mut [T],
) {
    //log!("TOP21: {}", top);
    let mut total_ratio: f32 = 0f32;

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        total_ratio += width as f32 / height as f32;
    }
    let optimal_height: f32 = max_width as f32 / total_ratio;
    let mut left: f32 = 0.0;

    for i in new_row_start..(new_row_end + 1) {
        // let line = String::new();
        let (width, height) = imgs[i].get_size();
        let new_width = optimal_height * (width as f32 / height as f32);
        let new_height = optimal_height;
        imgs[i].set_pos(left, *top, new_width, new_height);
        // imgs[i].new_width = optimal_height * (imgs[i].width as f64 / imgs[i].height as f64);
        // imgs[i].new_height = optimal_height;
        // imgs[i].left = left;
        // imgs[i].top = *top;
        //log!("{}:{:#?}", i,imgs[i].get_pos());
        left += new_width;
    }
   // log!("{:#?}", imgs);

    // let mut total: f64 = 0.0;
    // for i in new_row_start..(new_row_end + 1) {
    //     total += imgs[i].new_width;
    // }
    // log!("line: {}", total);

    *top += optimal_height;
    //log!("TOP22: {}", top);
}

pub fn resize_img2<T: GalleryImg + Debug>(
    top: &mut f32,
    max_width: u32,
    new_row_start: usize,
    new_row_end: usize,
    imgs: &mut [T],
) {
    //log!("TOP: {}", top);
    let mut optimal_count = (max_width as i32 / NEW_IMG_HEIGHT as i32) - (new_row_end - new_row_start)as i32;
    if optimal_count < 0 {
        optimal_count = 0;
    }
    let mut total_ratio: f32 = optimal_count as f32;
    if max_width < NEW_IMG_HEIGHT * 3 {
        total_ratio = 0.0;
    }


   // let mut total_ratio: f32 = 0.0;

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        total_ratio += width as f32 / height as f32;
    }
    let optimal_height: f32 = max_width as f32 / total_ratio;
    let mut left: f32 = 0.0;

    for i in new_row_start..(new_row_end + 1) {
        // let line = String::new();
        let (width, height) = imgs[i].get_size();
        let new_width = optimal_height * (width as f32 / height as f32);
        let new_height = optimal_height;
        imgs[i].set_pos(left, *top, new_width, new_height);
        // imgs[i].new_width = optimal_height * (imgs[i].width as f64 / imgs[i].height as f64);
        // imgs[i].new_height = optimal_height;
        // imgs[i].left = left;
        // imgs[i].top = *top;
        //log!("{}:{:#?}", i,imgs[i].get_pos());
        left += new_width;
    }


    // let mut total: f64 = 0.0;
    // for i in new_row_start..(new_row_end + 1) {
    //     total += imgs[i].new_width;
    // }
    // log!("line: {}", total);

    *top += optimal_height;
    //log!("TOP2: {}", top);
}


pub fn resize_imgs<T: GalleryImg + Debug>(new_height: u32, max_width: u32, imgs: &mut [T]) -> () {
    //log!("RESIZING!!!!!!!!!!!!");

    let loop_start = 0;
    let loop_end = imgs.len();
    //log!("resize: {} {} {:#?}", new_height, loop_end, imgs);
    let mut new_row_start: usize = 0;
    let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
    let mut current_row_filled_width: u32 = 0;
    // let new_height: u32 = NEW_IMG_HEIGHT;
    let mut top: f32 = 0.0;

    let mut rand = rand::thread_rng();
    for index in loop_start..loop_end {
        let org_img = &mut imgs[index];
        let (width, height) = org_img.get_size();
        // let width: u32 = org_img.width;
        // let height: u32 = org_img.height;
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: u32 = if height < new_height {
            0
        } else {
            height - new_height
        };
        let new_width: u32 = width - (height_diff as f32 * ratio) as u32;
        let id = rand.gen::<u128>();
        org_img.mark_as_modified(id);
        //log!("ADDING: {} {} {} {}", index, new_row_start, new_row_end, top);

        if (current_row_filled_width + new_width) <= max_width {
            //log!("REMOVING1: {} {} {} {}", index, new_row_start, new_row_end, top);

            current_row_filled_width += new_width;
            new_row_end = index;

            if index == loop_end - 1 {
                resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
        } else {
            if index != 0{
                //log!("REMOVING2: {} {} {} {}", index, new_row_start, new_row_end, top);
                resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
            if index == loop_end - 1 {
                resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
        }
    }
}

pub fn calc_fit_count(width: u32, height: u32) -> u32 {
    (width * height) / (NEW_IMG_HEIGHT * NEW_IMG_HEIGHT)
}

pub fn create_client_test_imgs() -> Vec<ServerMsgImgResized> {
    let mut new_imgs: Vec<ServerMsgImgResized> = Vec::new();
    for _ in 0..25 {
        new_imgs.push(ServerMsgImgResized::default());
    }
    new_imgs
}
