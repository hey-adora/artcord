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
    pub pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
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
            channels: StoredValue::new(HashMap::new()),
            global_on_open_callbacks: StoredValue::new(HashMap::new()),
            global_on_close_callbacks: StoredValue::new(HashMap::new()),
            global_on_ws_state_change_callbacks: StoredValue::new(HashMap::new()),
            pending_client_msgs: StoredValue::new(Vec::new()),
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
            let span_data = format!("ws({})", url);
            let _span = tracing::span!(tracing::Level::TRACE, "", "{}", span_data).entered();
            let _span = tracing::span!(tracing::Level::TRACE, "RUNTIME");

            let ws_on_msg = self.ws_on_msg;
            let ws_on_err = self.ws_on_err;
            let ws_on_open = self.ws_on_open;
            let ws_on_close = self.ws_on_close;
            let channels = self.channels;
            let ws_connected = self.connected;
            let ws_on_open_closures = self.global_on_open_callbacks;
            let ws_on_close_closures = self.global_on_close_callbacks;
            let ws_on_ws_state_closures = self.global_on_ws_state_change_callbacks;
            let ws_pending = self.pending_client_msgs;
            let ws = self.ws;

            ws_on_msg.set_value({
                let span_data = span_data.clone();
                Some(Rc::new(Closure::<dyn FnMut(_)>::new(
                    move |e: MessageEvent| {
                        let _span =
                            tracing::span!(tracing::Level::TRACE, "", "{}", span_data).entered();
                        let _span = tracing::span!(tracing::Level::TRACE, "RUNTIME-RECV");
                        Self::ws_on_msg(channels, e);
                    },
                )))
            });

            ws_on_err.set_value({
                let span_data = span_data.clone();
                Some(Rc::new(Closure::<dyn FnMut(_)>::new(
                    move |e: ErrorEvent| {
                        let _span =
                            tracing::span!(tracing::Level::TRACE, "", "{}", span_data).entered();
                        let _span = tracing::span!(tracing::Level::TRACE, "RUNTIME-ERR");
                        Self::ws_on_err(e);
                    },
                )))
            });

            ws_on_open.set_value({
                let span_data = span_data.clone();
                let ws_connected = ws_connected.clone();
                Some(Rc::new(Closure::<dyn FnMut()>::new(move || {
                    let _span =
                        tracing::span!(tracing::Level::TRACE, "", "{}", span_data).entered();
                    let _span = tracing::span!(tracing::Level::TRACE, "RUNTIME-OPEN");
                    Self::ws_on_open(ws, ws_connected, ws_pending, ws_on_ws_state_closures);
                })))
            });

            ws_on_close.set_value({
                let span_data = span_data.clone();
                let ws_connected = ws_connected.clone();
                Some(Rc::new(Closure::<dyn FnMut()>::new(move || {
                    let _span =
                        tracing::span!(tracing::Level::TRACE, "", "{}", span_data).entered();
                    let _span = tracing::span!(tracing::Level::TRACE, "RUNTIME-CLOSE");
                    Self::ws_on_close(ws, ws_connected, ws_on_ws_state_closures);
                })))
            });

            let create_ws = {
                move || -> WebSocket {
                    trace!("connecting...");
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
                    let span_data = span_data.clone();
                    move || {
                        let _span =
                            tracing::span!(tracing::Level::TRACE, "", "{}", span_data).entered();

                        let is_closed = ws.with_value(move |ws| {
                            ws.as_ref()
                                .and_then(|ws| Some(ws.ready_state() == WebSocket::CLOSED))
                                .unwrap_or(false)
                        });
                        if is_closed {
                            trace!("reconnecting...");
                            ws.set_value(Some(create_ws()));
                        }
                    }
                },
                1000,
            );

            let _timeout_interval = leptos_use::use_interval_fn(
                {
                    let span_data = span_data.clone();
                    move || {
                        let _span =
                            tracing::span!(tracing::Level::TRACE, "", "{}", span_data).entered();

                        tracing::trace_span!("TIMEOUT");

                        let is_closed = ws.with_value(move |ws| {
                            ws.as_ref()
                                .and_then(|ws| Some(ws.ready_state() != WebSocket::OPEN))
                                .unwrap_or(false)
                        });
                        if is_closed {
                            trace!("skipped, ws closed.",);
                            return;
                        }

                        let callbacks: Vec<(WsRouteKey, WsChannelCallbacksType<ServerMsg>)> =
                            channels
                                .try_update_value(|channels| {
                                    let mut output: Vec<(
                                        WsRouteKey,
                                        WsChannelCallbacksType<ServerMsg>,
                                    )> = Vec::new();

                                    for (i, (channel_key, channel)) in
                                        channels.iter_mut().enumerate()
                                    {
                                        let Some(delta) = channel.timeout_duratoin else {
                                            continue;
                                        };

                                        if channel.waiting_for_response == 0 {
                                            continue;
                                        }

                                        let Some(since) = channel.timeout_since else {
                                            channel.timeout_since = Some(Utc::now());
                                            trace!(
                                                "since date set: {:?} : {} : {:?} : {:?}",
                                                channel_key,
                                                channel.waiting_for_response,
                                                channel.timeout_since,
                                                channel.timeout_duratoin,
                                            );

                                            continue;
                                        };

                                        trace!(
                                            "comparing time: {:?} > {:?} & {}",
                                            Utc::now() - since,
                                            delta,
                                            channel.waiting_for_response
                                        );

                                        if Utc::now() - since > delta {
                                            trace!("found callback: {:?}", channel_key);
                                            output.push((
                                                channel_key.clone(),
                                                channel.callbacks.clone(),
                                            ));
                                        }
                                    }

                                    output
                                })
                                .unwrap_or_default();

                        for (channel_key, callbacks) in callbacks {
                            let _span = tracing::span!(
                                tracing::Level::TRACE,
                                "",
                                "{}",
                                format!("CHANNEL({:#01x})", channel_key)
                            )
                            .entered();

                            for (callback_key, callback) in callbacks {
                                let _span = tracing::span!(
                                    tracing::Level::TRACE,
                                    "",
                                    "{}",
                                    format!("CALLBACK({})", callback_key)
                                )
                                .entered();

                                let mut keep_open = true;
                                callback(&WsRecvResult::TimeOut, &mut keep_open);
                                Self::remove_callback(
                                    channels,
                                    channel_key,
                                    callback_key,
                                    keep_open,
                                );
                            }

                            Self::update_channel_after_recv(channels, channel_key);
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

    fn ws_on_open(
        ws: StoredValue<Option<WebSocket>>,
        connected: RwSignal<bool>,
        socket_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
        global_on_ws_state_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn(bool)>>>,
    ) {
        trace!(
            "connected, ws_on_closeclosures left {}",
            global_on_ws_state_callbacks.with_value(|c| c.len())
        );
        connected.set(true);
        Self::run_on_ws_state_callbacks(ws, global_on_ws_state_callbacks);
        Self::flush_pending_client_msgs(ws, socket_pending_client_msgs);
    }

    fn remove_callback(
        channels: WsChannelsType<ServerMsg>,
        channel_key: WsRouteKey,
        callback_key: WsRouteKey,
        keep_open: bool,
    ) {
        channels.update_value(|channels| {
            let Some(channel) = channels.get_mut(&channel_key) else {
                error!("channel not found",);
                return;
            };

            if !keep_open {
                let result = channel.callbacks.remove(&callback_key);
                if let Some(_) = result {
                    trace!("removed callback");
                } else {
                    error!("callback not found");
                }
            }
        });
    }

    fn update_channel_after_recv(channels: WsChannelsType<ServerMsg>, channel_key: WsRouteKey) {
        channels.update_value(|channels| {
            let Some(mut channel) = channels.get_mut(&channel_key) else {
                error!("channel not found");
                return;
            };

            Self::update_timout(channel);
        });
    }

    pub fn update_timout(channel: &mut WsChannelType<ServerMsg>) {
        tracing::trace_span!("TIMEOUT");

        let Some(value) = channel.waiting_for_response.checked_sub(1) else {
            warn!("received while not waiting");
            return;
        };
        channel.waiting_for_response = value;

        if channel.waiting_for_response > 0 {
            channel.timeout_since = Some(Utc::now());
        } else {
            channel.timeout_since = None;
        }
        trace!(
            "channel timeout state: {} {:?}",
            channel.waiting_for_response,
            channel.timeout_since
        );
    }

    pub fn channel(&self) -> ChannelBuilder<ServerMsg, ClientMsg> {
        ChannelBuilder::new(
            self.ws_url,
            self.channels,
            self.ws,
            self.pending_client_msgs,
            self.connected,
        )
    }

    fn ws_on_close(
        ws: StoredValue<Option<WebSocket>>,
        connected: RwSignal<bool>,
        global_on_ws_closure_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn(bool)>>>,
    ) {
        info!("disconnected");
        connected.set(false);
        Self::run_on_ws_state_callbacks(ws, global_on_ws_closure_callbacks);

        trace!(
            "callbacks left: {}",
            global_on_ws_closure_callbacks.with_value(|c| c.len())
        );
    }

    fn ws_on_err(e: ErrorEvent) {
        error!("{:?}", e);
    }

    fn ws_on_msg(channels: WsChannelsType<ServerMsg>, e: MessageEvent) {
        let data = e.data().dyn_into::<js_sys::ArrayBuffer>();
        let Ok(data) = data else {
            error!("failed to cast data");
            return;
        };
        let array = js_sys::Uint8Array::new(&data);
        let bytes: Vec<u8> = array.to_vec();

        if bytes.is_empty() {
            error!("is empty data");
            return;
        };

        let server_msg = ServerMsg::recv_from_vec(&bytes);
        let Ok(server_msg) = server_msg else {
            error!("data decoding: {}", server_msg.err().unwrap());
            return;
        };

        let _span = tracing::span!(
            tracing::Level::TRACE,
            "",
            "{}",
            format!("CHANNEL({:#01x})", server_msg.0)
        )
        .entered();

        trace!("data: \n{:#?}", &server_msg.1);

        Self::execute(channels, server_msg);
    }

    fn flush_pending_client_msgs(
        ws: StoredValue<Option<WebSocket>>,
        socket_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    ) {
        ws.with_value(|ws| {
            if let Some(ws) = ws {
                socket_pending_client_msgs.update_value(|msgs| {
                    trace!("sending from queue amount: {}", msgs.len());
                    let mut index: usize = 0;
                    for msg in msgs.iter() {
                        trace!("sending from queue {}: {:?}", index, msg);
                        let result = ws.send_with_u8_array(msg);
                        if result.is_err() {
                            warn!("failed to send {}: {:?}", index, msg);
                            break;
                        }

                        index += 1;
                    }
                    if index < msgs.len() && index > 0 {
                        *msgs = msgs[index..].to_vec();
                        warn!("msg left in queue: {}", msgs.len());
                    } else if index == msgs.len() {
                        *msgs = vec![];
                        trace!("msg left in queue is none: {}", msgs.len());
                    }
                });
            } else {
                warn!("not initialized.");
            }
        });
    }

    fn run_on_ws_state_callbacks(
        ws: StoredValue<Option<WebSocket>>,
        global_on_ws_state_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn(bool)>>>,
    ) {
        let is_connected = ws.with_value(|ws| {
            ws.as_ref()
                .map(|ws| ws.ready_state() == WebSocket::OPEN)
                .unwrap_or(false)
        });

        let callbacks = global_on_ws_state_callbacks.get_value();

        for (key, callback) in callbacks {
            let _span = tracing::span!(
                tracing::Level::TRACE,
                "",
                "{}",
                format!("STATE_CALLBACK({})", key)
            );

            trace!("running state callback");
            callback(is_connected);
        }
    }

    fn execute(channels: WsChannelsType<ServerMsg>, package: WsPackage<ServerMsg>) {
        let channel_key: WsRouteKey = package.0;
        let server_msg = WsRecvResult::Ok(package.1);

        let channel: Option<WsChannelType<ServerMsg>> = channels.with_value(move |channels| {
            let Some(f) = channels.get(&channel_key) else {
                error!("channel not found {:?}", &channel_key);
                return None;
            };

            Some(f.clone())
        });

        let Some(channel) = channel else {
            return;
        };

        for (callback_key, callback) in channel.callbacks {
            let _span = tracing::span!(
                tracing::Level::TRACE,
                "",
                "{}",
                format!("CALLBACK({})", callback_key)
            )
            .entered();
            trace!("running callback");

            let mut keep_open = true;
            callback(&server_msg, &mut keep_open);
            Self::remove_callback(channels, channel_key, callback_key, keep_open);
        }
        Self::update_channel_after_recv(channels, channel_key);
    }

    #[track_caller]
    pub fn on_ws_state(&self, callback: impl Fn(bool) + 'static) {
        let state_callback_key = crate::location_hash();

        let _span = tracing::span!(
            tracing::Level::TRACE,
            "",
            "{}",
            format!(
                "ws({})",
                self.ws_url.get_value().unwrap_or("error".to_string())
            )
        );

        let _span = tracing::span!(
            tracing::Level::TRACE,
            "",
            "{}",
            format!("STATE_CALLBACK({})", state_callback_key)
        );

        let is_connected = self.ws.with_value(|ws| {
            ws.as_ref()
                .map(|ws| ws.ready_state() == WebSocket::OPEN)
                .unwrap_or(false)
        });

        callback(is_connected);

        self.global_on_ws_state_change_callbacks.update_value({
            move |callbacks| {
                trace!("added state callback");
                callbacks.insert(state_callback_key, Rc::new(callback));
            }
        });

        on_cleanup({
            let callbacks = self.global_on_ws_state_change_callbacks;
            let ws_url = self.ws_url;
            move || {
                let _span = tracing::span!(
                    tracing::Level::TRACE,
                    "",
                    "{}",
                    format!("ws({})", ws_url.get_value().unwrap_or("error".to_string()))
                );

                let _span = tracing::span!(
                    tracing::Level::TRACE,
                    "",
                    "{}",
                    format!("STATE_CALLBACK({})", state_callback_key)
                );

                tracing::trace_span!("CLEANUP");

                callbacks.update_value({
                    move |callbacks| {
                        trace!("removed state callback");
                        callbacks.remove(&state_callback_key);
                    }
                });
            }
        });
    }
}
