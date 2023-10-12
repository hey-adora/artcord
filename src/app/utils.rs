use leptos::{create_rw_signal, window, RwSignal};
use wasm_bindgen::JsValue;
use web_sys::Location;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScrollSection {
    Home,
    About,
    Gallery,
}

#[derive(Copy, Clone, Debug)]
pub struct GlobalState {
    pub section: RwSignal<ScrollSection>,
    pub nav_open: RwSignal<bool>,
    pub nav_tran: RwSignal<bool>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            section: create_rw_signal(ScrollSection::Home),
            nav_open: create_rw_signal(false),
            nav_tran: create_rw_signal(true),
        }
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
