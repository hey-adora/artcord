use std::marker::PhantomData;
use std::num::{NonZeroU16, NonZeroU64};
use std::rc::Rc;
use std::time::Duration;
use std::{collections::HashMap, fmt::Debug};

use cfg_if::cfg_if;

use chrono::{DateTime, TimeDelta, Utc};
use leptos::*;
use leptos_use::use_window;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, trace, warn};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

const TIMEOUT_SECS: i64 = 30;

#[derive(Clone, Copy, Debug)]
pub enum WsOnRecvAction {
    RemoveCallback,
    RemoveTimeout,
    None,
}

#[derive(Clone)]
pub struct  WsChannelType <

    // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    ServerMsg: Clone + Receive + Debug + 'static,
    >{
    pub waiting_for_response: bool,
    // pub action: WsOnRecvAction,
    pub time: Option<(DateTime<chrono::Utc>, TimeDelta)>,
    pub callbacks: WsChannelCallbacksType<ServerMsg>,
    // pub phantom_temp_key: PhantomData<TempKeyType>,
    // pub phantom_perm_key: PhantomData<PermKeyType>,
}

impl <

    // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    ServerMsg: Clone + Receive + Debug + 'static,
    > WsChannelType<ServerMsg> {

    pub fn new(time: Option<(DateTime<chrono::Utc>, TimeDelta)>, callbacks: WsChannelCallbacksType<ServerMsg>) -> Self {
        Self {
            // action,
            time,
            callbacks,
            waiting_for_response: false,
            // phantom_temp_key: PhantomData,
            // phantom_perm_key: PhantomData,
        }
    }
}

pub type WsRouteKey = u128;
pub type WsPackage<ServerMsgType: Clone + 'static> = (WsRouteKey, ServerMsgType);
pub type WsChannelCallbacksType<ServerMsgType: Clone + 'static> = HashMap<WsRouteKey, Rc<dyn Fn(&WsRecvResult<ServerMsgType>) -> bool>>;
// #[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, Hash)]
// pub enum WsRouteKey<
//     TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
//     PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
// > {
//     Perm(PermKeyType),
//     GenPerm(TempKeyType),
//     TempSingle(TempKeyType),
// }

// #[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, Hash)]
// pub struct WsPackage<
//     // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
//     // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
//     Data: Clone + 'static,
// > {
//     pub key: WsRouteKey<TempKeyType, PermKeyType>,
//     pub data: Data,
// }

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

pub trait Send<
    // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
>
{
    fn send_as_vec(package: &WsPackage<Self>) -> Result<Vec<u8>, String>
    where
        Self: Clone;
}

pub trait Receive<
    // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
>
{
    fn recv_from_vec(bytes: &[u8]) -> Result<WsPackage<Self>, String>
    where
        Self: std::marker::Sized + Clone;
}

type WsCallbackType<T> = StoredValue<Option<Rc<Closure<T>>>>;
// type GlobalMsgCallbacksMulti<ServerMsg> = StoredValue<
//     HashMap<WsRouteKey, HashMap<WsRouteKey, Rc<dyn Fn(&ServerMsg)>>>,
// >;
type WsChannelsType<ServerMsg> = StoredValue<
    HashMap<
        WsRouteKey,
WsChannelType<ServerMsg>
    >,
>;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Clone, Debug)]
pub struct WsRuntime<
    // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + Send + Debug + 'static,
> {
    // pub global_msgs_callbacks_multi: GlobalMsgCallbacksMulti<TempKeyType, PermKeyType, ServerMsg>,
    pub channels: WsChannelsType<ServerMsg>,
    pub global_on_open_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn()>>>,
    pub global_on_close_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn()>>>,
    pub global_on_ws_state_change_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn(bool)>>>,
    pub global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    pub connected: RwSignal<bool>,
    pub ws: StoredValue<Option<WebSocket>>,
    pub ws_url: StoredValue<Option<String>>,
    pub ws_on_msg: WsCallbackType<dyn FnMut(MessageEvent)>,
    pub ws_on_err: WsCallbackType<dyn FnMut(ErrorEvent)>,
    pub ws_on_open: WsCallbackType<dyn FnMut()>,
    pub ws_on_close: WsCallbackType<dyn FnMut()>,
    phantom: PhantomData<ClientMsg>,
}

impl<
        // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + Send + Debug + 'static,
    > Copy for WsRuntime<ServerMsg, ClientMsg>
{
}

impl<
        // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + Send + Debug + 'static,
    > Default for WsRuntime<ServerMsg, ClientMsg>
{
    fn default() -> Self {
        Self {
            // global_msgs_callbacks_multi: StoredValue::new(HashMap::new()),
            channels: StoredValue::new(HashMap::new()),
            global_on_open_callbacks: StoredValue::new(HashMap::new()),
            global_on_close_callbacks: StoredValue::new(HashMap::new()),
            global_on_ws_state_change_callbacks: StoredValue::new(HashMap::new()),
            global_pending_client_msgs: StoredValue::new(Vec::new()),
            connected: RwSignal::new(false),
            ws: StoredValue::new(None),
            ws_url: StoredValue::new(None),
            ws_on_msg: StoredValue::new(None),
            ws_on_err: StoredValue::new(None),
            ws_on_open: StoredValue::new(None),
            ws_on_close: StoredValue::new(None),
            phantom: PhantomData,
        }
    }
}

impl<
        // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
        // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + Send + Debug + 'static,
    > WsRuntime<ServerMsg, ClientMsg>
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connect(&self, port: u32) -> Result<(), ConnectError> {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let path = get_ws_url(port)?;
                self.ws_url.set_value(Some(path.clone()));
                self.connect_to(&path);
            }
        }
        Ok(())
    }

    pub fn connect_to(&self, url: &str) {
        let connect = || {
            let url = String::from(url);

            let ws_on_msg = self.ws_on_msg;
            let ws_on_err = self.ws_on_err;
            let ws_on_open = self.ws_on_open;
            let ws_on_close = self.ws_on_close;
            // let ws_callbacks_multi = self.global_msgs_callbacks_multi;
            let channels = self.channels;
            let ws_connected = self.connected;
            //let ws_on_open_closures = self.global_on_open_callbacks;
            //let ws_on_close_closures = self.global_on_close_callbacks;
            let ws_on_ws_state_closures = self.global_on_ws_state_change_callbacks;
            let ws_pending = self.global_pending_client_msgs;
            let ws = self.ws;

            ws_on_msg.set_value({
                let url = url.clone();
                Some(Rc::new(Closure::<dyn FnMut(_)>::new(
                    move |e: MessageEvent| {
                        Self::ws_on_msg(&url, channels, e)
                    },
                )))
            });

            ws_on_err.set_value({
                let url = url.clone();
                Some(Rc::new(Closure::<dyn FnMut(_)>::new(
                    move |e: ErrorEvent| Self::ws_on_err(&url, e),
                )))
            });

            ws_on_open.set_value({
                let url = url.clone();
                let ws_connected = ws_connected.clone();
                Some(Rc::new(Closure::<dyn FnMut()>::new(move || {
                    Self::ws_on_open(ws, &url, ws_connected, ws_pending, ws_on_ws_state_closures)
                })))
            });

            ws_on_close.set_value({
                let url = url.clone();
                let ws_connected = ws_connected.clone();
                Some(Rc::new(Closure::<dyn FnMut()>::new(move || {
                    Self::ws_on_close(ws, &url, ws_connected, ws_on_ws_state_closures)
                })))
            });

            let create_ws = {
                let url = url.clone();
                move || -> WebSocket {
                    info!("ws({})_global: connecting", &url);
                    let ws = WebSocket::new(&url).unwrap();

                    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

                    ws_on_msg.with_value(|ws_on_msg| {
                        if let Some(ws_on_msg) = ws_on_msg {
                            ws.set_onmessage(Some((**ws_on_msg).as_ref().unchecked_ref()));
                        }
                    });

                    ws_on_err.with_value(|ws_on_err| {
                        if let Some(ws_on_err) = ws_on_err {
                            ws.set_onerror(Some((**ws_on_err).as_ref().unchecked_ref()));
                        }
                    });

                    ws_on_open.with_value(|ws_on_open| {
                        if let Some(ws_on_open) = ws_on_open {
                            ws.set_onopen(Some((**ws_on_open).as_ref().unchecked_ref()));
                        }
                    });

                    ws_on_close.with_value(|ws_on_close| {
                        if let Some(ws_on_close) = ws_on_close {
                            ws.set_onclose(Some((**ws_on_close).as_ref().unchecked_ref()));
                        }
                    });

                    ws
                }
            };

            ws.set_value(Some(create_ws()));
            let _reconnect_interval = leptos_use::use_interval_fn(
                {
                    let url = url.clone();
                    move || {
                        let is_closed = ws.with_value(move |ws| {
                            ws.as_ref()
                                .and_then(|ws| Some(ws.ready_state() == WebSocket::CLOSED))
                                .unwrap_or(false)
                        });
                        if is_closed {
                            info!("ws({}): reconnecting...", url);
                            ws.set_value(Some(create_ws()));
                        }
                    }
                },
                1000,
            );

            let _timeout_interval = leptos_use::use_interval_fn(
                {
                    let url = url.clone();
                    move || {
                        let callbacks: Vec<(WsRouteKey, WsChannelCallbacksType<ServerMsg>)> = channels.with_value(|channels| {
                            let mut output: Vec<(WsRouteKey, WsChannelCallbacksType<ServerMsg>)> = Vec::new();

                            // for i in 0..callbacks.len() {
                            //     let Some(item) = callbacks.get(i) else {
                            //         break;
                            //     };
                            // }
                            //trace!("ws({}): timeout: total callback count: {}", url, callbacks.len());
                            for (i, (channel_key, channel)) in channels.iter().enumerate() {
                                if !channel.waiting_for_response {
                                    continue;
                                }
                        
                                if let Some((time, delta)) = channel.time {
                                    trace!("ws({}): timedout: comparing time: {:?} > {:?} & {}", url,  Utc::now() - time, delta, channel.waiting_for_response);
                                    if Utc::now() - time > TimeDelta::microseconds(TIMEOUT_SECS * 1000 * 1000) {
                                        trace!("ws({}): timedout: found callback: {:?}", url, channel_key);
                                        output.push((channel_key.clone(), channel.callbacks.clone()));

                                    } else {
                                        trace!("ws({}): timeout: finished looking for callbacks at: {}", url, i);
                                        break;
                                    }
                                }
                            }

                            output
                        });

                        // let Some(callbacks) = callbacks else {
                        //     trace!("ws({}): timeout: no callbacks found", url);
                        //     return;
                        // };

                        //trace!("ws({}): timeout: timedout callback count: {}", url, callbacks.len());

                        for (channel_key, callbacks) in callbacks {
                            trace!("ws({}): timeout: running callback: {:?}", url, &channel_key);
                            for (callback_key, callback) in callbacks {
                                trace!("1111111111wtf, run run run!");
                                let keep_open = callback(&WsRecvResult::TimeOut);

                                trace!("wtf, run run run!");
                                Self::update_callback_after_recv(channels, &url, channel_key, callback_key, keep_open);
                            }
                            Self::update_channel_after_recv(channels, &url, channel_key);
                        }
                    }
                },
                1000,
            );

        };
        #[cfg(target_arch = "wasm32")]
        {
            connect();
        }
    }

    fn update_callback_after_recv(channels: WsChannelsType<ServerMsg>, url: &str, channel_key: WsRouteKey, callback_key: WsRouteKey, keep_open: bool) {
        trace!("ws({}): updating channel after recv...: {:?}", url, &channel_key);
        channels.update_value(|channels| {
            let Some(channel) = channels.get_mut(&channel_key) else {
                warn!("ws({}): channel after recv not found: {:?}", url, &channel_key);
                return;
            };

            if !keep_open {
                let result = channel.callbacks.remove(&callback_key);
                if let Some(result) = result {
                    trace!("ws({}): removed callback: {:?}", url, &callback_key);
                } else {
                    warn!("ws({}): callback not found: {:?}", url, &callback_key);
                }
            }

            trace!("ws({}): waiting_for_response set to false: {:?}", url, &callback_key);
            channel.waiting_for_response = false;


        });

    }

    fn update_channel_after_recv(channels: WsChannelsType<ServerMsg>, url: &str, channel_key: WsRouteKey) {
        trace!("ws({}): updating channel after recv...{}", url, channel_key);
        channels.update_value(|channels| {
            let Some(channel) = channels.get_mut(&channel_key) else {
                warn!("ws({}): channel after recv not found: {:?}", url, &channel_key);
                return;
            };

            trace!("ws({}): waiting_for_response set to false", url);
            channel.waiting_for_response = false;
        });

    }

    // fn update_channel_after_recv()

    pub fn builder(&self) -> WsBuilder<ServerMsg, ClientMsg> {
        WsBuilder::new(
            self.ws_url,
            self.channels,
            self.ws,
            self.global_pending_client_msgs,
        )
    }

    // pub fn create_node(&self) -> WsPortal<TempKeyType, PermKeyType, ServerMsg, ClientMsg> {
    //     WsPortal::<TempKeyType, PermKeyType, ServerMsg, ClientMsg>::new(
    //         self.ws_url,
    //         self.global_msgs_callbacks_single,
    //         self.ws,
    //         self.global_pending_client_msgs,
    //     )
    // }

    fn ws_on_open(
        ws: StoredValue<Option<WebSocket>>,
        url: &str,
        connected: RwSignal<bool>,
        socket_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
        global_on_ws_state_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn(bool)>>>,
    ) {
        info!(
            "ws({})_global: connected, ws_on_closeclosures left {}",
            url,
            global_on_ws_state_callbacks.with_value(|c| c.len())
        );
        connected.set(true);
        Self::run_on_ws_state_callbacks(ws, url, global_on_ws_state_callbacks);
        //Self::run_on_open_callbacks(url, global_on_open_callbacks);
        Self::flush_pending_client_msgs(ws, url, socket_pending_client_msgs);
    }

    fn ws_on_close(
        ws: StoredValue<Option<WebSocket>>,
        url: &str,
        connected: RwSignal<bool>,
        global_on_ws_closure_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn(bool)>>>,
    ) {
        info!("ws({})_global: disconnected", url);
        connected.set(false);
        Self::run_on_ws_state_callbacks(ws, url, global_on_ws_closure_callbacks);
        //Self::run_on_ws_state_callbacks(ws, url, global_on_ws_closure_callbacks);
        trace!(
            "ws({})_global: disconnect: ws_on_closeclosures left: {}",
            url,
            global_on_ws_closure_callbacks.with_value(|c| c.len())
        );
    }

    fn ws_on_err(url: &str, e: ErrorEvent) {
        error!("WS({})_global: error: {:?}", url, e);
    }

    fn ws_on_msg(
        url: &str,
        // callbacks_multi: GlobalMsgCallbacksMulti<TempKeyType, PermKeyType, ServerMsg>,
        callbacks_single: WsChannelsType<ServerMsg>,
        e: MessageEvent,
    ) {
        let data = e.data().dyn_into::<js_sys::ArrayBuffer>();
        let Ok(data) = data else {
            return;
        };
        let array = js_sys::Uint8Array::new(&data);
        let bytes: Vec<u8> = array.to_vec();

        if bytes.is_empty() {
            trace!("ws({})_global: recv empty msg.", url);
            return;
        };

        let server_msg = ServerMsg::recv_from_vec(&bytes);
        let Ok(server_msg) = server_msg else {
            error!(
                "ws({})_global: error decoding msg: {}",
                url,
                server_msg.err().unwrap()
            );
            return;
        };

        debug!("ONE ONE ONE ");
        trace!("ws({})_global: recved msg: {:#?}", url, &server_msg);

        Self::execute(url, callbacks_single, server_msg);
        debug!("TWO TWO TWO ");
        // match &server_msg.key {
        //     WsRouteKey::Perm(_) => {
        //         Self::execute_multi(url, callbacks_multi, server_msg);
        //     }
        //
        //     WsRouteKey::TempSingle(_) | WsRouteKey::GenPerm(_) => {
        //     }
        // }
    }

    fn flush_pending_client_msgs(
        ws: StoredValue<Option<WebSocket>>,
        url: &str,
        socket_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    ) {
        ws.with_value(|ws| {
            if let Some(ws) = ws {
                socket_pending_client_msgs.update_value(|msgs| {
                    trace!(
                        "ws({})_global: sending msgs from queue, left: {}",
                        url,
                        msgs.len()
                    );
                    let mut index: usize = 0;
                    for msg in msgs.iter() {
                        trace!(
                            "ws({})_global: sending from msg {} from queue: {:?}",
                            url,
                            index,
                            msg
                        );
                        let result = ws.send_with_u8_array(msg);
                        if result.is_err() {
                            warn!("ws({})_global: failed to send msg {}:{:?}", url, index, msg);
                            break;
                        }

                        index += 1;
                    }
                    if index < msgs.len() && index > 0 {
                        *msgs = (&msgs[index..]).to_vec();
                        trace!("ws({})_global: msg left in queue: {}", url, msgs.len());
                    }
                });
            } else {
                warn!("ws({})_global: not initialized.", url);
            }
        });
    }

    // let is_connected = self.ws.with_value(|ws| {
    //     ws.as_ref().map(|ws|ws.ready_state() == WebSocket::CLOSED).unwrap_or(false)
    // });

    fn run_on_ws_state_callbacks(
        ws: StoredValue<Option<WebSocket>>,
        url: &str,
        global_on_ws_state_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn(bool)>>>,
    ) {
        //debug!("3420 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

        let is_connected = ws.with_value(|ws| {
            ws.as_ref()
                .map(|ws| ws.ready_state() == WebSocket::OPEN)
                .unwrap_or(false)
        });

        let callbacks = global_on_ws_state_callbacks.get_value();
        for (key, callback) in callbacks {
            trace!(
                "ws({})_global: running on_ws_state callback: {:#?}",
                url,
                key
            );
            callback(is_connected);
        }
    }

    // fn run_on_open_callbacks(
    //     url: &str,
    //     global_on_open_callbacks: StoredValue<HashMap<TempKeyType, Rc<dyn Fn()>>>,
    // ) {
    //     let callbacks = global_on_open_callbacks.get_value();
    //     for (key, callback) in callbacks {
    //         trace!("ws({})_global: running on_open callback: {:#?}", url, key);
    //         callback();
    //     }
    // }

    // fn run_on_close_callbacks(
    //     url: &str,
    //     global_on_close_callbacks: StoredValue<HashMap<TempKeyType, Rc<dyn Fn()>>>,
    // ) {
    //     let callbacks = global_on_close_callbacks.get_value();
    //     for (key, callback) in callbacks {
    //         trace!("ws({})_global: running on_close callback: {:#?}", url, key);
    //         callback();
    //     }
    // }

    // fn execute_multi(
    //     url: &str,
    //     callbacks_multi: GlobalMsgCallbacks<ServerMsg>,
    //     package: WsPackage<ServerMsg>,
    // ) {
    //     let key: WsRouteKey = package.0.clone();
    //
    //     let callback_cluster: Option<HashMap<TempKeyType, Rc<dyn Fn(&ServerMsg)>>> =
    //         callbacks_multi.with_value({
    //             let key = key.clone();
    //
    //             move |socket_closures| {
    //                 let Some(f) = socket_closures.get(&key) else {
    //                     warn!("ws({})_global: Fn not found for {:?}", url, &key);
    //                     return None;
    //                 };
    //
    //                 Some(f.clone())
    //             }
    //         });
    //
    //     let Some(callback_cluster) = callback_cluster else {
    //         return;
    //     };
    //
    //     match &key {
    //         WsRouteKey::Perm(_) => {
    //             for (key, callback) in callback_cluster {
    //                 trace!(
    //                     "ws({})_global: running(execute_multi) callback: {:#?}",
    //                     url,
    //                     key
    //                 );
    //                 callback(&package.data);
    //             }
    //         }
    //         _ => {
    //             warn!("ws({})_global: Wrong key was selected: {:?}", url, &key);
    //         }
    //     }
    // }

    fn execute(
        url: &str,
        channels: WsChannelsType<ServerMsg>,
        package: WsPackage<ServerMsg>,
    ) {
        let channel_key: WsRouteKey = package.0;
        let server_msg = WsRecvResult::Ok(package.1);

        let channel: Option<WsChannelType<ServerMsg>> = channels.with_value(
            move |channels| {
                let Some(f) = channels.get(&channel_key) else {
                    warn!("ws({})_global: channel not found {:?}", url, &channel_key);
                    return None;
                };

                Some(f.clone())
            }
);

        let Some(channel) = channel else {
            return;
        };


        debug!("THREE THREE");

        for (callback_key, callback) in channel.callbacks {
            trace!(
                "ws({})_global: running(execute_single) callback: {:#?}",
                url, channel_key
            );

            debug!("FOUR FOUR");
            let keep_open = callback(&server_msg);
            debug!("FIVE FIVE");
            Self::update_callback_after_recv(channels, &url, channel_key, callback_key, keep_open);
            debug!("SIX SIX");
            // channels.update_value(|channels| {
            //     let Some(channel) = channels.get_mut(&channel_key) else {
            //         warn!("ws({}): cleanup fail: channel not found: {}", url, channel_key);
            //         return;
            //     };
            //
            //     if !keep_open {
            //         let removed = channel.callbacks.remove(&callback_key).is_some();
            //         if !removed {
            //             warn!("ws({}): cleanup fail: callback not found: {}", url, callback_key);
            //         } else {}
            //     }
            // });
        }
        Self::update_channel_after_recv(channels, &url, channel_key);




        // match &key {
        //     WsRouteKey::TempSingle(_) => {
        //         trace!(
        //             "ws({})_global: running(execute_single) callback: {:#?}",
        //             url,
        //             key
        //         );
        //         (ws_callback.callback)(WsResourceResult::Ok(package.data));
        //         callbacks_single.update_value(|callbacks| {
        //             let result = callbacks.remove(&key);
        //             if let Some(result) = result {
        //                 trace!("ws({})_global: execute_single callback was removed: {:?}", url, &key);
        //             } else {
        //                 warn!("ws({})_global: execute_single failed to remove callback: current_key: {:?}, current callbacks: {:#?}", url, &key, callbacks_single.get_value().keys());
        //             }
        //         });
        //     }
        //     WsRouteKey::GenPerm(_) => {
        //         trace!(
        //             "ws({})_global: running(execute_single) callback: {:#?}",
        //             url,
        //             key
        //         );
        //         (ws_callback.callback)(WsResourceResult::Ok(package.data));
        //         callbacks_single.update_value(|callbacks| {
        //             let result = callbacks.get_mut(&key);
        //             if let Some(result) = result {
        //                 result.time = None;
        //                 trace!("ws({})_global: execute_single timeout was removed: {:?}", url, &key);
        //             } else {
        //                 warn!("ws({})_global: execute_single failed to remove timeout: current_key: {:?}, current callbacks: {:#?}", url, &key, callbacks_single.get_value().keys());
        //             }
        //         });
        //
        //     }
        //     _ => {
        //         warn!("ws({})_global: Wrong key was selected: {:?}", url, &key);
        //     }
        // }
    }

    // reimplement in protal
    // pub fn send(
    //     &self,
    //     perm_key: PermKeyType,
    //     client_msg: ClientMsg,
    // ) -> Result<SendResult, WsError> {
    //     self.ws.with_value(|ws| -> Result<SendResult, WsError> {
    //         let package = WsPackage::<TempKeyType, PermKeyType, ClientMsg> {
    //             data: client_msg,
    //             key: WsRouteKey::Perm(perm_key),
    //         };
    //         let bytes = ClientMsg::send_as_vec(&package).map_err(WsError::Serializatoin)?;
    //
    //         if let Some(ws) = ws {
    //             let is_open = self.ws.with_value(move |ws| {
    //                 ws.as_ref()
    //                     .map(|ws| ws.ready_state() == WebSocket::OPEN)
    //                     .unwrap_or(false)
    //             });
    //
    //             if is_open {
    //                 trace!(
    //                     "ws({})_global: msg \"{:?}\" sending",
    //                     self.ws_url.get_value().unwrap_or("error".to_string()),
    //                     &package
    //                 );
    //
    //                 return ws
    //                     .send_with_u8_array(&bytes)
    //                     .map(|_| SendResult::Sent)
    //                     .map_err(|err| {
    //                         error!(
    //                             "ws({}): failed to send: {:?}",
    //                             self.ws_url.get_value().unwrap_or("error".to_string()),
    //                             &err
    //                         );
    //                         WsError::SendError(
    //                             err.as_string()
    //                                 .unwrap_or(String::from("Failed to send web-socket package")),
    //                         )
    //                     });
    //             }
    //         }
    //
    //         trace!(
    //             "ws({})_global: msg \"{:?}\" pushed to queue",
    //             self.ws_url.get_value().unwrap_or("error".to_string()),
    //             &package
    //         );
    //         self.global_pending_client_msgs
    //             .update_value(|pending| pending.push(bytes));
    //         Ok(SendResult::Queued)
    //     })
    // }

    // pub fn on_ws_state() {

    // }

    pub fn on_ws_state(&self, callback: impl Fn(bool) + 'static) {
        //console_log!("3420 count this");
        let temp_key: WsRouteKey = u128::generate_key();

        let is_connected = self.ws.with_value(|ws| {
            ws.as_ref()
                .map(|ws| ws.ready_state() == WebSocket::OPEN)
                .unwrap_or(false)
        });

        callback(is_connected);

        self.global_on_ws_state_change_callbacks.update_value({
            let temp_key = temp_key.clone();
            move |callbacks| {
                trace!(
                    "ws({})_global: adding on_ws_state callback: {:#?}",
                    self.ws_url.get_value().unwrap_or("error".to_string()),
                    temp_key
                );
                callbacks.insert(temp_key, Rc::new(callback));
            }
        });

        on_cleanup({
            let callbacks = self.global_on_ws_state_change_callbacks;
            let ws_url = self.ws_url;
            move || {
                callbacks.update_value({
                    move |callbacks| {
                        trace!(
                            "ws({})_global: cleanup: removing on_ws_state callback: {:#?}",
                            ws_url.get_value().unwrap_or("error".to_string()),
                            temp_key
                        );
                        callbacks.remove(&temp_key);
                    }
                });
            }
        });
    }

    // pub fn on_open(&self, callback: impl Fn() + 'static) {
    //     let temp_key = TempKeyType::generate_key();

    //     self.global_on_open_callbacks.update_value({
    //         let temp_key = temp_key.clone();
    //         move |callbacks| {
    //             trace!(
    //                 "ws({})_global: adding on_open callback: {:#?}",
    //                 self.ws_url.get_value().unwrap_or("error".to_string()),
    //                 temp_key
    //             );
    //             callbacks.insert(temp_key, Rc::new(callback));
    //         }
    //     });

    //     on_cleanup({
    //         let callbacks = self.global_on_open_callbacks;
    //         let ws_url = self.ws_url;
    //         move || {
    //             callbacks.update_value({
    //                 move |callbacks| {
    //                     trace!(
    //                         "ws({})_global: cleanup: removing on_open callback: {:#?}",
    //                         ws_url.get_value().unwrap_or("error".to_string()),
    //                         temp_key
    //                     );
    //                     callbacks.remove(&temp_key);
    //                 }
    //             });
    //         }
    //     });
    // }

    // pub fn on_close(&self, callback: impl Fn() + 'static) {
    //     let temp_key = TempKeyType::generate_key();

    //     self.global_on_close_callbacks.update_value({
    //         let temp_key = temp_key.clone();
    //         move |callbacks| {
    //             trace!(
    //                 "ws({})_global: adding on_close callback: {:#?}",
    //                 self.ws_url.get_value().unwrap_or("error".to_string()),
    //                 temp_key
    //             );
    //             callbacks.insert(temp_key, Rc::new(callback));
    //         }
    //     });

    //     on_cleanup({
    //         let callbacks = self.global_on_close_callbacks;
    //         let ws_url = self.ws_url;
    //         move || {
    //             callbacks.update_value({
    //                 move |callbacks| {
    //                     trace!(
    //                         "ws({})_global: cleanup: removing on_close callback: {:#?}",
    //                         ws_url.get_value().unwrap_or("error".to_string()),
    //                         temp_key
    //                     );
    //                     callbacks.remove(&temp_key);
    //                 }
    //             });
    //         }
    //     });
    // }


    // re-implement in portal
//     pub fn on(&self, perm_key: PermKeyType, on_receive: impl Fn(&ServerMsg) + 'static) {
//         let perm_key = WsRouteKey::<TempKeyType, PermKeyType>::Perm(perm_key);
//         let temp_key = TempKeyType::generate_key();
//         self.global_msgs_callbacks_multi.update_value({
//             let temp_key = temp_key.clone();
//             let perm_key = perm_key.clone();
//             move |global_msgs_callbacks| {
//                 let current_callback_cluster = global_msgs_callbacks.get_mut(&perm_key);
//
//                 match current_callback_cluster {
//                     Some(current_callback_cluster) => {
//                         trace!(
//                             "ws({})_global: adding global_msgs_closures callback: {:#?}",
//                             self.ws_url.get_value().unwrap_or("error".to_string()),
//                             temp_key
//                         );
//                         current_callback_cluster.insert(temp_key, Rc::new(on_receive));
//                     }
//                     None => {
//                         let mut callback_cluster: HashMap<TempKeyType, Rc<dyn Fn(&ServerMsg)>> =
//                             HashMap::new();
//                         trace!(
//                             "ws({})_global: adding global_msgs_closures callback: {:#?}",
//                             self.ws_url.get_value().unwrap_or("error".to_string()),
//                             temp_key
//                         );
//                         callback_cluster.insert(temp_key, Rc::new(on_receive));
//                         global_msgs_callbacks.insert(perm_key, callback_cluster);
//                     }
//                 }
//             }
//         });
//         on_cleanup({
//             let callbacks = self.global_msgs_callbacks_multi;
//             let ws_url = self.ws_url;
//             move || {
//                 callbacks.update_value({
//                     move |callbacks| {
//                         let callback_cluster = callbacks.get_mut(&perm_key);
//                         let Some(callback_cluster) = callback_cluster else {
//                             trace!(
//                                 "ws({})_global: cleanup: closure not found: {:?}",
//                                 ws_url.get_value().unwrap_or("error".to_string()),
//                                 &perm_key
//                             );
//                             return;
//                         };
//                         trace!(
//                             "ws({})_global: cleanup: removed: {:?}",
//                             ws_url.get_value().unwrap_or("error".to_string()),
//                             &perm_key
//                         );
//                         callback_cluster.remove(&temp_key);
//                     }
//                 });
//             }
//         });
//     }
// }
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

#[derive(Clone, Debug, Copy)]
pub struct WsBuilder<
    // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + Send + Debug + 'static,
> {
    ws_url: StoredValue<Option<String>>,
    global_msgs_closures: WsChannelsType<ServerMsg>,
    ws: StoredValue<Option<WebSocket>>,
    global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    phantom: PhantomData<ClientMsg>,
}

impl<
        // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
        // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + Send + Debug + 'static,
    > WsBuilder<ServerMsg, ClientMsg>
{
    pub fn new(
        ws_url: StoredValue<Option<String>>,
        global_msgs_closures: WsChannelsType<ServerMsg>,
        ws: StoredValue<Option<WebSocket>>,
        global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    ) -> Self {
        Self {
            ws_url,
            global_msgs_closures,
            ws,
            global_pending_client_msgs,
            phantom: PhantomData,
        }
    }

    pub fn channel_with_timeout(self, timeout_secs: i64) -> ChannelBuilder<ServerMsg, ClientMsg> {
        ChannelBuilder {
            ws_builder: self,
            skip_if_awaiting_response: true,
            timeout: Some(TimeDelta::microseconds(timeout_secs * 1000 * 1000)),
        }
    }
}
//TimeDelta::microseconds(TIMEOUT_SECS * 1000 * 1000)
#[derive(Clone, Debug, Copy)]
pub struct ChannelBuilder<
    // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + Send + Debug + 'static,
> {
    ws_builder: WsBuilder<ServerMsg, ClientMsg>,
    skip_if_awaiting_response: bool,
    timeout: Option<TimeDelta>,
}

impl<
        // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
        // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + Send + Debug + 'static,
    > ChannelBuilder<ServerMsg, ClientMsg>
{
    // pub fn stream(mut self) -> Self {
    //     self.on_recv_action = WsOnRecvAction::None;
    //     self
    // }


    pub fn multi(mut self) -> Self {
        self.skip_if_awaiting_response = false;
        self
    }

    pub fn build(self) -> WsChannel<ServerMsg, ClientMsg> {
        WsChannel::new(
            self.ws_builder.ws_url,
            self.ws_builder.global_msgs_closures,
            self.ws_builder.ws,
            self.ws_builder.global_pending_client_msgs,
            self.skip_if_awaiting_response,
            self.timeout,
        )
    }
}

// #[derive(Clone, Debug, Copy)]
// pub struct WsBuilder {
//     settings: WsResourceSettings
// }
//
// pub fn

#[track_caller]
fn location_hash() -> u128 {
    xxhash_rust::xxh3::xxh3_128(std::panic::Location::caller().to_string().as_bytes())
    // fasthash::spooky::hash128()
}

#[derive(Clone, Debug)]
pub struct WsChannel<
    // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + Send + Debug + 'static,
> {
    channel: WsChannelsType<ServerMsg>,
    global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    //socket_send_fn: StoredValue<Rc<dyn Fn(Vec<u8>)>>,
    ws: StoredValue<Option<WebSocket>>,
    ws_url: StoredValue<Option<String>>,
    key: WsRouteKey,
    phantom: PhantomData<ClientMsg>,
    skip_if_awaiting_response: bool,
    timeout: Option<TimeDelta>,
    // settings: WsBuilder,
}



impl<
        // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + Send + Debug + 'static,
    > Copy for WsChannel<ServerMsg, ClientMsg>
{
}

impl<
        // TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        // PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + Send + Debug + 'static,
    > WsChannel<ServerMsg, ClientMsg>
{
    pub fn new(
        ws_url: StoredValue<Option<String>>,
        global_msgs_closures: WsChannelsType<ServerMsg>,
        ws: StoredValue<Option<WebSocket>>,
        global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
        skip_if_awaiting_response: bool,
        timeout: Option<TimeDelta>,
        // settings: WsBuilder,
    ) -> Self {
        // let ws_round_kind = if single_use {
        //     WsRouteKey::<TempKeyType, PermKeyType>::TempSingle(TempKeyType::generate_key())
        // } else {
        //     WsRouteKey::<TempKeyType, PermKeyType>::GenPerm(TempKeyType::generate_key())
        // };
        let channel_key = WsRouteKey::generate_key();

        let create_channel = || global_msgs_closures.update_value({
            move |channels| {
                    let Some(channel) = channels.get_mut(&channel_key) else {
                        let mut channel_callbacks: WsChannelCallbacksType<ServerMsg> = HashMap::new();
                        let time = Some((
                                        Utc::now(),
                                        TimeDelta::microseconds(TIMEOUT_SECS * 1000 * 1000),
                                    ));

                        let channel = WsChannelType::new(time , channel_callbacks) ;
                        channels.insert(channel_key, channel);
                        return;
                    };
                    warn!("ws({}): channel already exists: {}", ws_url.get_value().unwrap_or("error".to_string()), channel_key);
            }
        });

        #[cfg(target_arch = "wasm32")]
        {
            create_channel();
        }

        let detach = Owner::current().is_none();
        if !detach {
            on_cleanup({
                // let key = ws_round_kind.clone();
                move || {
                    global_msgs_closures.update_value({
                        move |socket_closures| {
                            trace!(
                                "ws({})_global: channel removed: {:?}",
                                ws_url.get_value().unwrap_or("error".to_string()),
                                &channel_key
                            );
                            socket_closures.remove(&channel_key);
                        }
                    });
                }
            });
        }

        Self {
            channel: global_msgs_closures,
            global_pending_client_msgs,
            ws,
            ws_url,
            key: channel_key,
            phantom: PhantomData,
            // on_recv_action,
            // detach,
            skip_if_awaiting_response,
            timeout,
            // settings,
        }
    }

    #[track_caller]
    pub fn recv(
        &self,
        on_receive: impl Fn(&WsRecvResult<ServerMsg>) -> bool + 'static,

    ) {

        let channel_key = self.key;
        let callback_key = location_hash();
        // let action = self.on_recv_action;
        self.ws.with_value(|ws| -> Result<(), WsError> {
            self.channel.update_value({
                move |channels| {
                    let Some(channel) = channels.get_mut(&channel_key) else {
                        trace!(
                            "ws({}): channel was not created: {}",
                            self.ws_url.get_value().unwrap_or("error".to_string()),
                            &callback_key);
                        return;
                    };

                        
                    let contains = channel
                            .callbacks.contains_key(&callback_key);
                    if !contains {
                        trace!(
                            "ws({})_global: adding global_msgs_closures callback: {:#?}",
                            self.ws_url.get_value().unwrap_or("error".to_string()),
                            &callback_key);
                        channel.callbacks.insert(channel_key, Rc::new(on_receive));

                    } 
                        
                }
            });

            Ok(())
        });
    }

    pub fn send(&self, client_msg: ClientMsg) {
        let channel_key = self.key;
        self.ws
            .with_value(|ws| -> Result<WsResourcSendResult, WsError> {
                if self.skip_if_awaiting_response {
                    let waiting_for_response = self.channel.with_value(|channels| channels.get(&channel_key).map(|channel|channel.waiting_for_response).unwrap_or(false));
                    if waiting_for_response {
                        return Ok(WsResourcSendResult::Skipped);
                    }
                }
                let package: WsPackage<ClientMsg> = (channel_key, client_msg);
                // {
                //     data: client_msg,
                //     key: self.key.get_value(),
                // };

                let bytes = ClientMsg::send_as_vec(&package).map_err(WsError::Serializatoin)?;

                if let Some(ws) = ws {
                    let is_open = self.ws.with_value(move |ws| {
                        ws.as_ref()
                            .map(|ws| ws.ready_state() == WebSocket::OPEN)
                            .unwrap_or(false)
                    });

                    if is_open {
                        trace!(
                            "ws({}): sending msg: {:?}",
                            self.ws_url.get_value().unwrap_or("error".to_string()),
                            &package
                        );
                        return ws
                            .send_with_u8_array(&bytes)
                            .map(|_| {
                                self.set_is_waiting_for_response();
                                WsResourcSendResult::Sent
                            })
                            .map_err(|err| {
                                WsError::SendError(
                                    err.as_string().unwrap_or(String::from(
                                        "Failed to send web-socket package",
                                    )),
                                )
                            });
                    }
                }

                trace!(
                    "ws({}): msg \"{:?}\" pushed to queue",
                    self.ws_url.get_value().unwrap_or("error".to_string()),
                    &package
                );

                self.global_pending_client_msgs
                    .update_value(|pending| pending.push(bytes));


                self.set_is_waiting_for_response();
                Ok(WsResourcSendResult::Queued)
                // let waiting_for_response = self.channel.with_value(|callback| {
                //     // callback.w
                // });

                // Ok(())
                

            });

    }

    fn set_is_waiting_for_response(&self) {
        self.channel.update_value(|channels| {
            let Some(channel) = channels.get_mut(&self.key) else {
                warn!(
                    "ws({}): cant set waiting for response, channel not found: {}",
                    self.ws_url.get_value().unwrap_or("error".to_string()),
                    &self.key
                );
                return;
            };
            trace!(
                "ws({}): waiting for response enabled for: {}",
                self.ws_url.get_value().unwrap_or("error".to_string()),
                &self.key
            );
            channel.waiting_for_response = true;
        });
    }

    // #[track_caller]
    // pub fn send_and_recv(
    //     &self,
    //     client_msg: ClientMsg,
    //     on_receive: impl Fn(&WsRecvResult<ServerMsg>) -> bool + 'static,
    // ) -> Result<WsResourcSendResult, WsError> {
    //     let channel_key = self.key;
    //     let callback_key = location_hash();
    //     let action = self.on_recv_action;
    //     self.ws
    //         .with_value(|ws| -> Result<WsResourcSendResult, WsError> {
    //             let exists: Option<bool> = self.channel.try_update_value({
    //                 move |channels| {
    //
    //
    //                         let Some(channel) = channels.get_mut(&channel_key) else {
    //                             let mut channel_callbacks: WsChannelCallbacksType<ServerMsg> = HashMap::new();
    //                             channel_callbacks.insert(callback_key, Rc::new(on_receive));
    //                             let time = Some((
    //                                             Utc::now(),
    //                                             TimeDelta::microseconds(TIMEOUT_SECS * 1000 * 1000),
    //                                         ));
    //
    //                             let channel = WsChannelType::new(action, time , channel_callbacks) ;
    //                             channels.insert(channel_key, channel);
    //                             return false;
    //                         };
    //
    //
    //                     let contains = channel
    //                             .callbacks.contains_key(&callback_key);
    //                     if contains {
    //                         trace!(
    //                             "ws({})_global: adding global_msgs_closures callback: {:#?}",
    //                             self.ws_url.get_value().unwrap_or("error".to_string()),
    //                             &callback_key);
    //                         channel.callbacks.insert(channel_key, Rc::new(on_receive));
    //
    //                     } 
    //                     contains
    //
    //                 }
    //             });
    //
    //             if exists.unwrap_or(false) {
    //                 trace!(
    //                     "ws({}): skipping: callback already exists: {:?}",
    //                     self.ws_url.get_value().unwrap_or("error".to_string()),
    //                     self.key
    //                 );
    //                 return Ok(WsResourcSendResult::Skipped);
    //             }
    //
    //             let package: WsPackage<ClientMsg> = (channel_key, client_msg);
    //             // {
    //             //     data: client_msg,
    //             //     key: self.key.get_value(),
    //             // };
    //
    //             let bytes = ClientMsg::send_as_vec(&package).map_err(|err| {
    //                 self.remove_callback(callback_key);
    //                 WsError::Serializatoin(err)
    //             })?;
    //
    //             if let Some(ws) = ws {
    //                 let is_open = self.ws.with_value(move |ws| {
    //                     ws.as_ref()
    //                         .map(|ws| ws.ready_state() == WebSocket::OPEN)
    //                         .unwrap_or(false)
    //                 });
    //
    //                 if is_open {
    //                     trace!(
    //                         "ws({}): sending msg: {:?}",
    //                         self.ws_url.get_value().unwrap_or("error".to_string()),
    //                         &package
    //                     );
    //                     return ws
    //                         .send_with_u8_array(&bytes)
    //                         .map(|_| WsResourcSendResult::Sent)
    //                         .map_err(|err| {
    //                             self.remove_callback(callback_key);
    //                             WsError::SendError(
    //                                 err.as_string().unwrap_or(String::from(
    //                                     "Failed to send web-socket package",
    //                                 )),
    //                             )
    //                         });
    //                 }
    //             }
    //
    //             trace!(
    //                 "ws({}): msg \"{:?}\" pushed to queue",
    //                 self.ws_url.get_value().unwrap_or("error".to_string()),
    //                 &package
    //             );
    //
    //             self.global_pending_client_msgs
    //                 .update_value(|pending| pending.push(bytes));
    //
    //             Ok(WsResourcSendResult::Queued)
    //         })
    // }

    fn remove_callback(&self, callback_key: u128) -> bool {
        let channel_key = self.key;
        // let a = async {

        // };
        // let b = tokio::spawn(a);

        let mut output = false;
        self.channel.update_value({
            |socket_closures| {
                let channel = socket_closures.get_mut(&channel_key);
                let Some(channel) = channel else {

                    error!(
                        "ws({}): error: cant find channel with key: {:?}",
                        self.ws_url.get_value().unwrap_or("error".to_string()),
                        &channel_key
                    );
                    return;
                };


                output = channel.callbacks.remove(&callback_key).is_some();
                if output {
                    trace!(
                        "ws({}): channel callback removed {:?}",
                        self.ws_url.get_value().unwrap_or("error".to_string()),
                        &callback_key
                    );
                 } else {
                    error!(
                        "ws({}): error: cant find callback with key: {:?}",
                        self.ws_url.get_value().unwrap_or("error".to_string()),
                        &callback_key
                    );

                }
                    //                 error!(
                    //     "ws({}): error: cant find callback with key: {:?}",
                    //     self.ws_url.get_value().unwrap_or("error".to_string()),
                    //     &channel_key
                    // );

                // if  {
                //
                //     output = true;
                // } else {
                // }
            }
        });
        output
    }
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

#[derive(Debug, Clone)]
pub enum WsResourcSendResult {
    Sent,
    Skipped,
    Queued,
}

#[derive(Debug, Clone)]
pub enum SendResult {
    Sent,
    Queued,
}

#[derive(Error, Debug, Clone)]
pub enum WsError {
    #[error("Sending error: {0}.")]
    SendError(String),

    #[error("Failed to serialize client message: {0}.")]
    Serializatoin(String),
}

#[derive(Debug, Clone)]
pub enum WsRecvResult<T: Debug + Clone> {
    Ok(T),
    TimeOut,
}
