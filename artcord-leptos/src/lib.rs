pub mod app;

use app::App;
use wasm_bindgen::prelude::wasm_bindgen;
use leptos::{logging::log, *};

#[wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount_to_body(App)
}


// use crate::leptos_dom::mount_to_with_stop_hydrating; a

// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = console)]
//     fn log(s: &str);
// }

// #[wasm_bindgen]
// pub fn hydrate() {
//     _ = console_log::init_with_level(log::Level::Debug);
//     console_error_panic_hook::set_once();
//     log!("HHHHHHHHHHHHHHHHHHHHHH");
//     console_log!("oh ok?");
//     //leptos::mount_to_body(App);
//     mount_to_with_stop_hydrating(document().body().expect("body element to exist"), true, App)
// }
// use wasm_bindgen::prelude::wasm_bindgen;

// #[wasm_bindgen]
// pub fn hydrate() {
//     console_error_panic_hook::set_once();

//     leptos::mount_to_body(App);
// }


// use cfg_if::cfg_if;

// cfg_if! {
// if #[cfg(feature = "hydrate")] {

//     use wasm_bindgen::prelude::wasm_bindgen;

//         #[wasm_bindgen]
//         pub fn hydrate() {
//         console_error_panic_hook::set_once();

//         leptos::mount_to_body(App);
//         }
//     }
// }