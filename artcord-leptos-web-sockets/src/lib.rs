use std::fmt::Debug;
use std::marker::PhantomData;

use chrono::TimeDelta;
use leptos::*;
use leptos_use::use_window;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;
use wasm_bindgen::JsCast;
use web_sys::WebSocket;

pub mod channel;
pub mod channel_builder;
pub mod runtime;

pub const TIMEOUT_SECS: i64 = 30;

// #[derive(Clone, Copy, Debug)]
// pub enum WsOnRecvAction {
//     RemoveCallback,
//     RemoveTimeout,
//     None,
// }

pub type WsRouteKey = u128;
pub type WsPackage<MsgType: Clone + 'static> = (WsRouteKey, MsgType);

impl KeyGen for u128 {
    fn generate_key() -> Self {
        uuid::Uuid::new_v4().as_u128()
    }
}

impl KeyGen for String {
    fn generate_key() -> Self {
        uuid::Uuid::new_v4().to_string()
    }
}

pub trait KeyGen
where
    Self: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
{
    fn generate_key() -> Self;
}

pub trait Send {
    fn send_as_vec(package: &WsPackage<Self>) -> Result<Vec<u8>, String>
    where
        Self: Clone;
}

pub trait Receive {
    fn recv_from_vec(bytes: &[u8]) -> Result<WsPackage<Self>, String>
    where
        Self: std::marker::Sized + Clone;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub fn get_ws_url(port: u32) -> Result<String, GetUrlError> {
    let mut output = String::new();
    let window = use_window();
    let window = window.as_ref().ok_or(GetUrlError::GetWindow)?;

    let protocol = window
        .location()
        .protocol()
        .or(Err(GetUrlError::GetProtocol))?;

    if protocol == "http:" {
        output.push_str("ws://");
    } else {
        output.push_str("wss://");
    }

    let hostname = window
        .location()
        .hostname()
        .or(Err(GetUrlError::GetHostname))?;
    output.push_str(&format!("{}:{}", hostname, port));

    Ok(output)
}
use wasm_bindgen::prelude::wasm_bindgen;

#[track_caller]
fn location_hash() -> u128 {
    xxhash_rust::xxh3::xxh3_128(std::panic::Location::caller().to_string().as_bytes())
}

#[derive(Error, Debug)]
pub enum ConnectError {
    #[error("Failed to get generate connection url: {0}")]
    GetUrlError(#[from] GetUrlError),
}

#[derive(Error, Debug)]
pub enum GetUrlError {
    #[error("UseWindow() returned None")]
    GetWindow,

    #[error("window.location().protocol() failed")]
    GetProtocol,

    #[error("window.location().hostname() failed")]
    GetHostname,
}

#[derive(Error, Debug, Clone)]
pub enum WsError {
    #[error("Sending error: {0}.")]
    SendError(String),

    #[error("Failed to serialize client message: {0}.")]
    Serializatoin(String),
}
