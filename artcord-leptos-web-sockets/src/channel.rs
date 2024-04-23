use crate::{KeyGen, Receive, Send, WsError, WsRouteKey, TIMEOUT_SECS};
use chrono::{DateTime, TimeDelta, Utc};
use leptos::{create_effect, on_cleanup, Owner, RwSignal, SignalGet, StoredValue};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use tracing::{error, trace, warn};
use wasm_bindgen::closure::Closure;
use web_sys::WebSocket;

pub mod channel_with_timeout;

#[derive(Clone)]
pub struct WsChannelType<ServerMsg: Clone + Receive + Debug + 'static> {
    pub waiting_for_response: u32,
    pub timeout_duratoin: Option<TimeDelta>,
    pub timeout_since: Option<(DateTime<chrono::Utc>)>,
    pub callbacks: WsChannelCallbacksType<ServerMsg>,
}

impl<ServerMsg: Clone + Receive + Debug + 'static> WsChannelType<ServerMsg> {
    pub fn new(
        timeout_duratoin: Option<TimeDelta>,
        callbacks: WsChannelCallbacksType<ServerMsg>,
    ) -> Self {
        Self {
            timeout_since: None,
            callbacks,
            waiting_for_response: 0,
            timeout_duratoin,
        }
    }
}

pub type WsChannelCallbacksType<ServerMsgType: Clone + 'static> =
    HashMap<u128, Rc<dyn Fn(&WsRecvResult<ServerMsgType>, &mut bool)>>;

pub type WsCallbackType<T> = StoredValue<Option<Rc<Closure<T>>>>;
pub type WsChannelsType<ServerMsg> = StoredValue<HashMap<u128, WsChannelType<ServerMsg>>>;

#[derive(Clone, Debug)]
pub struct WsChannel<
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + crate::Send + Debug + 'static,
> {
    channels: WsChannelsType<ServerMsg>,
    global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    ws: StoredValue<Option<WebSocket>>,
    ws_url: StoredValue<Option<String>>,
    key: u128,
    phantom: PhantomData<ClientMsg>,
    single_fire: bool,
    timeout: Option<TimeDelta>,
    is_connected: RwSignal<bool>,
}

impl<
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + crate::Send + Debug + 'static,
    > Copy for WsChannel<ServerMsg, ClientMsg>
{
}

impl<
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + crate::Send + Debug + 'static,
    > WsChannel<ServerMsg, ClientMsg>
{
    #[track_caller]
    pub fn new(
        ws_url: StoredValue<Option<String>>,
        channels: WsChannelsType<ServerMsg>,
        ws: StoredValue<Option<WebSocket>>,
        global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
        single_fire: bool,
        timeout: Option<TimeDelta>,
        persistant: bool,
        is_connected: RwSignal<bool>,
        key: Option<WsRouteKey>,
    ) -> Self {
        let channel_key = if let Some(key) = key {
            key
        } else {
            crate::location_hash()
        };

        let create_channel = || {
            channels.update_value({
                move |channels| {
                    let Some(channel) = channels.get_mut(&channel_key) else {
                        channels.insert(channel_key, WsChannelType::new(timeout, HashMap::new()));
                        trace!(
                            "ws({}): channel inserted: {}",
                            ws_url.get_value().unwrap_or("error".to_string()),
                            channel_key
                        );
                        return;
                    };

                    if !persistant {
                        *channel = WsChannelType::new(timeout, HashMap::new());
                        trace!(
                            "ws({}): channel replaced: {}",
                            ws_url.get_value().unwrap_or("error".to_string()),
                            channel_key
                        );
                    } else {
                        trace!(
                            "ws({}): channel already exists: {}",
                            ws_url.get_value().unwrap_or("error".to_string()),
                            channel_key
                        );
                    }
                }
            })
        };

        #[cfg(target_arch = "wasm32")]
        {
            create_channel();
        }

        if !persistant {
            on_cleanup({
                move || {
                    channels.update_value({
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
            channels,
            global_pending_client_msgs,
            ws,
            ws_url,
            key: channel_key,
            phantom: PhantomData,
            single_fire,
            timeout,
            is_connected,
        }
    }

    #[track_caller]
    pub fn start_recv(
        &self,
        on_receive: impl Fn(&WsRecvResult<ServerMsg>, &mut bool) + 'static,
        persistant: bool,
        // key: Option<u128>,
    ) {
        let channel_key = self.key;
        let callback_key = crate::location_hash();
        self.ws.with_value(|ws| {
            self.channels.update_value({
                move |channels| {
                    let Some(channel) = channels.get_mut(&channel_key) else {
                        trace!(
                            "ws({}): channel was not created: {}",
                            self.ws_url.get_value().unwrap_or("error".to_string()),
                            &callback_key
                        );
                        return;
                    };

                    let contains = channel.callbacks.contains_key(&callback_key);
                    if !contains {
                        trace!(
                            "ws({})_global: adding global_msgs_closures callback: {:#?}",
                            self.ws_url.get_value().unwrap_or("error".to_string()),
                            &callback_key
                        );
                        channel.callbacks.insert(channel_key, Rc::new(on_receive));
                    } else if !persistant {
                        trace!(
                            "ws({})_global: replacing global_msgs_closures callback: {:#?}",
                            self.ws_url.get_value().unwrap_or("error".to_string()),
                            &callback_key
                        );
                        channel.callbacks.insert(channel_key, Rc::new(on_receive));
                    } else {
                        trace!(
                            "ws({})_global: persistant global_msgs_closures callback: {:#?}",
                            self.ws_url.get_value().unwrap_or("error".to_string()),
                            &callback_key
                        );
                    }
                }
            });
        });

        if !persistant {
            let channels = self.channels;
            let ws_url = self.ws_url;
            on_cleanup({
                move || {
                    channels.update_value({
                        move |socket_closures| {
                            trace!(
                                "ws({})_global: channel '{}' callback removed: {}",
                                ws_url.get_value().unwrap_or("error".to_string()),
                                &channel_key,
                                &callback_key,
                            );

                            let channel = socket_closures.get_mut(&channel_key);
                            let Some(channel) = channel else {
                                return;
                            };
                            channel.callbacks.remove(&callback_key);
                        }
                    });
                }
            });
        }
    }

    #[track_caller]
    pub fn send(
        &self,
        client_msg: ClientMsg,
        on_cleanup_msg: Option<ClientMsg>,
        resend_on_reconnect: bool,
        // last_msg: bool
    ) -> Result<WsResourcSendResult, WsError> {
        let owner = Owner::current();
        if owner.is_none() {
            let mut errors: Option<String> = None;

            let mut add_err = |err: &str| {
                if let Some(errors) = &mut errors {
                    errors.push_str(err);
                } else {
                    errors = Some(String::from(err));
                }
            };

            if on_cleanup_msg.is_some() {
                add_err("on_cleanup_msg cant run outside reactive system.\n");
            }

            if resend_on_reconnect {
                add_err("on_cleanup_msg cant run outside reactive system.\n");
            }

            if let Some(errors) = errors {
                let location = std::panic::Location::caller().to_string();
                warn!("ws send error at {}\n{}", location, errors);
            }
        }

        let channel_key = self.key;
        if let Some(client_msg) = on_cleanup_msg {
            let channel = self.clone();
            // let send = self.send;
            on_cleanup(move || {
                let result = channel.send(client_msg, None, false);
                if let Err(err) = result {
                    warn!(
                        "ws({})_send: error on cleanup '{}' : {}",
                        channel.ws_url.get_value().unwrap_or("error".to_string()),
                        &channel_key,
                        err
                    );
                }
            });
        }

        if resend_on_reconnect {
            let channel = self.clone();
            // let client_msg = client_msg.clone();
            // self.ws.with_value(|ws| {
            //     if let Some(ws) = ws {
            //         ws.stat
            //     }
            // });
            create_effect(move |_| {
                let is_connected = channel.is_connected.get();
                if !is_connected {
                    return;
                }
                let result = channel.send(client_msg.clone(), None, false);
                if let Err(err) = result {
                    warn!(
                        "ws({})_send: error on connect '{}' : {}",
                        channel.ws_url.get_value().unwrap_or("error".to_string()),
                        &channel_key,
                        err
                    );
                }
            });
            return Ok(WsResourcSendResult::EventAdded);
        }

        self.ws
            .with_value(|ws| -> Result<WsResourcSendResult, WsError> {
                if self.single_fire {
                    let waiting_for_response = self.channels.with_value(|channels| {
                        channels
                            .get(&channel_key)
                            .map(|channel| channel.waiting_for_response)
                            .unwrap_or(0)
                    });
                    if waiting_for_response > 0 {
                        return Ok(WsResourcSendResult::Skipped);
                    }
                }
                let package: (u128, ClientMsg) = (channel_key, client_msg);

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
                                self.update_timeout();
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

                self.update_timeout();
                Ok(WsResourcSendResult::Queued)
            })
    }

    fn update_timeout(&self) {
        self.channels.update_value(|channels| {
            let Some(channel) = channels.get_mut(&self.key) else {
                trace!(
                    "ws({}): cant set waiting for response, channel not found: {}",
                    self.ws_url.get_value().unwrap_or("error".to_string()),
                    &self.key
                );
                return;
            };
            channel.waiting_for_response += 1;
            // if channel.timeout_since.is_none() {
            //     channel.timeout_since = Some(Utc::now());
            // }
            trace!(
                "ws({}): waiting for response enabled for: {} : {} : {:?} : {:?}",
                self.ws_url.get_value().unwrap_or("error".to_string()),
                &self.key,
                channel.waiting_for_response,
                channel.timeout_since,
                channel.timeout_duratoin,
            );
        });
    }

    fn remove_callback(&self, callback_key: u128) -> bool {
        let channel_key = self.key;
        let mut output = false;

        self.channels.update_value({
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
            }
        });
        output
    }
}

#[derive(Debug, Clone)]
pub enum WsResourcSendResult {
    Sent,
    Skipped,
    Queued,
    EventAdded,
}

#[derive(Debug, Clone)]
pub enum SendResult {
    Sent,
    Queued,
}

#[derive(Debug, Clone)]
pub enum WsRecvResult<T: Debug + Clone> {
    Ok(T),
    TimeOut,
}
