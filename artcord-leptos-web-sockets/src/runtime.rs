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

use crate::channel::{
    WsCallbackType, WsChannelCallbacksType, WsChannelType, WsChannelsType, WsRecvResult,
};
use crate::channel_builder::ChannelBuilder;
use crate::{get_ws_url, ConnectError, KeyGen, Receive, Send, WsPackage, WsRouteKey, TIMEOUT_SECS};

#[derive(Clone, Debug)]
pub struct WsRuntime<
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + Send + Debug + 'static,
> {
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

impl<ServerMsg: Clone + Receive + Debug + 'static, ClientMsg: Clone + Send + Debug + 'static> Copy
    for WsRuntime<ServerMsg, ClientMsg>
{
}

impl<ServerMsg: Clone + Receive + Debug + 'static, ClientMsg: Clone + Send + Debug + 'static>
    Default for WsRuntime<ServerMsg, ClientMsg>
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

impl<ServerMsg: Clone + Receive + Debug + 'static, ClientMsg: Clone + Send + Debug + 'static>
    WsRuntime<ServerMsg, ClientMsg>
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connect(&self, port: u32) -> Result<(), ConnectError> {
        let connect = || -> Result<(), ConnectError> {
            let path = get_ws_url(port)?;
            self.ws_url.set_value(Some(path.clone()));
            self.connect_to(&path);
            Ok(())
        };

        #[cfg(target_arch = "wasm32")]
        {
            connect()?;
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
            let channels = self.channels;
            let ws_connected = self.connected;
            let ws_on_open_closures = self.global_on_open_callbacks;
            let ws_on_close_closures = self.global_on_close_callbacks;
            let ws_on_ws_state_closures = self.global_on_ws_state_change_callbacks;
            let ws_pending = self.global_pending_client_msgs;
            let ws = self.ws;

            ws_on_msg.set_value({
                let url = url.clone();
                Some(Rc::new(Closure::<dyn FnMut(_)>::new(
                    move |e: MessageEvent| Self::ws_on_msg(&url, channels, e),
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
                        let is_closed = ws.with_value(move |ws| {
                            ws.as_ref()
                                .and_then(|ws| Some(ws.ready_state() != WebSocket::OPEN))
                                .unwrap_or(false)
                        });
                        if is_closed {
                            trace!("ws({}): timedout: skipped, ws closed.", url,);
                            return;
                        }

                        let callbacks: Vec<(WsRouteKey, WsChannelCallbacksType<ServerMsg>)> =
                            channels.try_update_value(|channels| {

                                let mut output: Vec<(
                                    WsRouteKey,
                                    WsChannelCallbacksType<ServerMsg>,
                                )> = Vec::new();

                                for (i, (channel_key, channel)) in channels.iter_mut().enumerate() {
                                    let Some(delta) = channel.timeout_duratoin else {
                                        continue;
                                    };

                                    if channel.waiting_for_response == 0 {
                                        continue;
                                    }

                                    let Some(since) = channel.timeout_since else {
                                        channel.timeout_since = Some(Utc::now());
                                        trace!(
                                            "ws({}): timedout: since date set: {:?} : {} : {:?} : {:?}",
                                            url, channel_key,
                                            channel.waiting_for_response,
                                            channel.timeout_since,
                                            channel.timeout_duratoin,

                                        );

                                        continue;
                                    };

                                    trace!(
                                        "ws({}): timedout: comparing time: {:?} > {:?} & {}",
                                        url,
                                        Utc::now() - since,
                                        delta,
                                        channel.waiting_for_response
                                    );

                                    if Utc::now() - since > delta {
                                        trace!(
                                            "ws({}): timedout: found callback: {:?}",
                                            url,
                                            channel_key
                                        );
                                        output
                                            .push((channel_key.clone(), channel.callbacks.clone()));
                                    }
                                    // else {
                                    //     trace!("ws({}): timeout: finished looking for callbacks at: {}", url, i);
                                    //     break;
                                    // }
                                }

                                output
                            }).unwrap_or_default();

                        for (channel_key, callbacks) in callbacks {
                            trace!("ws({}): timeout: running callback: {:?}", url, &channel_key);
                            for (callback_key, callback) in callbacks {
                                // trace!("1111111111wtf, run run run!");
                                let keep_open = callback(&WsRecvResult::TimeOut);

                                // trace!("wtf, run run run!");
                                Self::update_callback_after_recv(
                                    channels,
                                    &url,
                                    channel_key,
                                    callback_key,
                                    keep_open,
                                );
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

    pub fn on_open(&self, callback: impl Fn() + 'static) {
        let temp_key = u128::generate_key();

        self.global_on_open_callbacks.update_value({
            let temp_key = temp_key.clone();
            move |callbacks| {
                trace!(
                    "ws({})_global: adding on_open callback: {:#?}",
                    self.ws_url.get_value().unwrap_or("error".to_string()),
                    temp_key
                );
                callbacks.insert(temp_key, Rc::new(callback));
            }
        });

        on_cleanup({
            let callbacks = self.global_on_open_callbacks;
            let ws_url = self.ws_url;
            move || {
                callbacks.update_value({
                    move |callbacks| {
                        trace!(
                            "ws({})_global: cleanup: removing on_open callback: {:#?}",
                            ws_url.get_value().unwrap_or("error".to_string()),
                            temp_key
                        );
                        callbacks.remove(&temp_key);
                    }
                });
            }
        });
    }

    fn update_callback_after_recv(
        channels: WsChannelsType<ServerMsg>,
        url: &str,
        channel_key: WsRouteKey,
        callback_key: WsRouteKey,
        keep_open: bool,
    ) {
        trace!(
            "ws({}): updating callbacks after recv...: {:?}",
            url,
            &channel_key
        );
        channels.update_value(|channels| {
            let Some(channel) = channels.get_mut(&channel_key) else {
                warn!(
                    "ws({}): channel after recv not found: {:?}",
                    url, &channel_key
                );
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
        });
    }

    fn update_channel_after_recv(
        channels: WsChannelsType<ServerMsg>,
        url: &str,
        channel_key: WsRouteKey,
    ) {
        trace!("ws({}): updating channel after recv...{}", url, channel_key);
        channels.update_value(|channels| {
            let Some(mut channel) = channels.get_mut(&channel_key) else {
                warn!(
                    "ws({}): channel after recv not found: {:?}",
                    url, &channel_key
                );
                return;
            };

            Self::update_timout(url, channel);
        });
    }

    pub fn update_timout(url: &str, channel: &mut WsChannelType<ServerMsg>) {
        let Some(value) = channel.waiting_for_response.checked_sub(1) else {
            error!("failed to subtract response");
            return;
        };
        channel.waiting_for_response = value;

        if channel.waiting_for_response > 0 {
            channel.timeout_since = Some(Utc::now());
        } else {
            channel.timeout_since = None;
        }
        trace!(
            "ws({}): state after updating channel: {} {:?}",
            url,
            channel.waiting_for_response,
            channel.timeout_since
        );
    }

    pub fn channel(&self) -> ChannelBuilder<ServerMsg, ClientMsg> {
        ChannelBuilder::new(
            self.ws_url,
            self.channels,
            self.ws,
            self.global_pending_client_msgs,
        )
    }

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

        trace!(
            "ws({})_global: disconnect: ws_on_closeclosures left: {}",
            url,
            global_on_ws_closure_callbacks.with_value(|c| c.len())
        );
    }

    fn ws_on_err(url: &str, e: ErrorEvent) {
        error!("WS({})_global: error: {:?}", url, e);
    }

    fn ws_on_msg(url: &str, channels: WsChannelsType<ServerMsg>, e: MessageEvent) {
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

        Self::execute(url, channels, server_msg);
        debug!("TWO TWO TWO ");
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
                        *msgs = msgs[index..].to_vec();
                        warn!("ws({})_global: msg left in queue: {}", url, msgs.len());
                    } else if index == msgs.len() {
                        *msgs = vec![];
                        trace!(
                            "ws({})_global: msg left in queue is none: {}",
                            url,
                            msgs.len()
                        );
                    }
                });
            } else {
                warn!("ws({})_global: not initialized.", url);
            }
        });
    }

    fn run_on_ws_state_callbacks(
        ws: StoredValue<Option<WebSocket>>,
        url: &str,
        global_on_ws_state_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn(bool)>>>,
    ) {
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

    fn execute(url: &str, channels: WsChannelsType<ServerMsg>, package: WsPackage<ServerMsg>) {
        let channel_key: WsRouteKey = package.0;
        let server_msg = WsRecvResult::Ok(package.1);

        let channel: Option<WsChannelType<ServerMsg>> = channels.with_value(move |channels| {
            let Some(f) = channels.get(&channel_key) else {
                warn!("ws({})_global: channel not found {:?}", url, &channel_key);
                return None;
            };

            Some(f.clone())
        });

        let Some(channel) = channel else {
            return;
        };

        debug!("THREE THREE");

        for (callback_key, callback) in channel.callbacks {
            trace!(
                "ws({})_global: running(execute_single) callback: {:#?}",
                url,
                channel_key
            );

            debug!("FOUR FOUR");
            let keep_open = callback(&server_msg);
            debug!("FIVE FIVE");
            Self::update_callback_after_recv(channels, &url, channel_key, callback_key, keep_open);
            debug!("SIX SIX");
        }
        Self::update_channel_after_recv(channels, &url, channel_key);
    }

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
}
