use artcord_state::aggregation::server_msg_img::AggImg;
use artcord_state::misc::img_quality::ImgQuality;
use artcord_state::model::user::User;
use chrono::Utc;
use leptos::*;
use leptos::{window, RwSignal, SignalGetUntracked};
use leptos_router::use_location;
use regex::Regex;
use std::fmt::{Debug, Display};
use std::ops;
use tracing::{debug, error, trace};
use wasm_bindgen::JsValue;
use web_sys::Location;

use crate::app::pages::admin::ws_old::PAGE_AMOUNT;

use self::img_resize::GalleryImg;

use super::global_state::GlobalState;

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
pub enum PageUrl {
    Home,
    HomeAbout,
    MainGallery,
    UserGallery,
    AdminDash,
    AdminDashWsLive,
    AdminDashWsOld,
    AdminThrottleCached,
    NotFound,
}

// struct Vg

impl Display for PageUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PageUrl::Home => "/",
                PageUrl::HomeAbout => "/#about",
                PageUrl::MainGallery => "/gallery",
                PageUrl::UserGallery => "/user/:id",
                PageUrl::AdminDash => "/dash",
                PageUrl::AdminDashWsLive => "/wslive",
                PageUrl::AdminDashWsOld => "/wsold",
                PageUrl::AdminThrottleCached => "/throttle_cached",
                PageUrl::NotFound => "/*any",
            }
        )
    }
}

// impl std::ops::Fn<()> for PageUrl {
//     type Output = Fn();
//     fn call(&self, args: Args) -> Self::Output {}
// }

impl PageUrl {
    pub fn url_home() -> String {
        PageUrl::Home.to_string()
    }

    pub fn url_home_about() -> String {
        PageUrl::HomeAbout.to_string()
    }

    pub fn url_main_gallery() -> String {
        PageUrl::MainGallery.to_string()
    }
    // /user/:id
    pub fn url_user_gallery(user_id: &str) -> String {
        format!("/user/{}", user_id)
    }

    pub fn url_dash() -> String {
        PageUrl::AdminDash.to_string()
    }

    pub fn url_throttle_cached() -> String {
        format!("{}{}", PageUrl::AdminDash, PageUrl::AdminThrottleCached)
    }

    pub fn url_dash_wslive() -> String {
        format!("{}{}", PageUrl::AdminDash, PageUrl::AdminDashWsLive)
    }

    pub fn url_dash_wsold() -> String {
        format!("{}{}", PageUrl::AdminDash, PageUrl::AdminDashWsOld)
    }

    pub fn url_dash_wsold_paged(page: u64, from: i64) -> String {
        format!("{}{}?p={}&a={}&f={}", PageUrl::AdminDash, PageUrl::AdminDashWsOld, page, PAGE_AMOUNT, from)
    }

    pub fn url_dash_wsold_refresh(page: u64) -> String {
        format!("{}{}?p={}&a={}", PageUrl::AdminDash, PageUrl::AdminDashWsOld, page, PAGE_AMOUNT)
    }

    pub fn update_current_page_url() {
        let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
        let re = move |value: &str, re: &str| -> bool {
            Regex::new(re)
                .inspect_err(|err| {
                    error!("url regex error: {}", err);
                })
                .map(|re| re.captures_iter(value).next().is_some())
                .unwrap_or(false)
        };
        create_effect(move |_| {
            let location = use_location();

            // let mut a = re.it

            let url = format!("{}{}", location.pathname.get(), location.hash.get());
            let url = url.as_str();
            let url = match url {
                url if url == Self::url_home() || url == Self::url_home() => PageUrl::Home,
                url if url == Self::url_home_about() => PageUrl::HomeAbout,
                url if url == Self::url_main_gallery() => PageUrl::MainGallery,
                url if url == Self::url_dash() => PageUrl::AdminDash,
                url if url == Self::url_dash_wslive() => PageUrl::AdminDashWsLive,
                url if url == Self::url_dash_wsold() => PageUrl::AdminDashWsOld,
                url if url == Self::url_throttle_cached() => PageUrl::AdminThrottleCached,
                url if re(url, r"^\/user\/[[:alnum:]]+$") => PageUrl::UserGallery,
                _ => PageUrl::NotFound,
            };

            trace!("current url: {}", url);

            if url != global_state.current_page_url.get() {
                global_state.current_page_url.set(url);
            }

            // let section: PageUrl =
            //     match  {
            //         "/gallery" => PageUrl::MainGallery,
            //         "/#about" => PageUrl::HomeAbout,
            //         s if s.contains("/user/") => PageUrl::UserProfile,
            //         _ => PageUrl::NotFound,
            //     };
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
