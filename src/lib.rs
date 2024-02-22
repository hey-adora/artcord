#![allow(unused_variables, unused_imports)]

pub mod app;
pub mod bot;
pub mod database;
pub mod message;
pub mod server;

use crate::app::App;
use cfg_if::cfg_if;
use leptos::*;

cfg_if! {
if #[cfg(feature = "hydrate")] {

  use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn hydrate() {
      console_error_panic_hook::set_once();

      leptos::mount_to_body(App);
    }
}
}
