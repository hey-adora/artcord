use crate::{KeyGen, Receive, Send, WsError, WsRouteKey, TIMEOUT_SECS};
use chrono::{DateTime, TimeDelta, Utc};
use leptos::{on_cleanup, Owner, StoredValue};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use tracing::{error, trace, warn};
use wasm_bindgen::closure::Closure;
use web_sys::WebSocket;

#[derive(Clone)]
pub struct WsChannelType<ServerMsg: Clone + Receive + Debug + 'static> {
    pub waiting_for_response: bool,
    pub time: Option<(DateTime<chrono::Utc>, TimeDelta)>,
    pub callbacks: WsChannelCallbacksType<ServerMsg>,
}

impl<ServerMsg: Clone + Receive + Debug + 'static> WsChannelType<ServerMsg> {
    pub fn new(
        time: Option<(DateTime<chrono::Utc>, TimeDelta)>,
        callbacks: WsChannelCallbacksType<ServerMsg>,
    ) -> Self {
        Self {
            time,
            callbacks,
            waiting_for_response: false,
        }
    }
}

pub type WsChannelCallbacksType<ServerMsgType: Clone + 'static> =
    HashMap<WsRouteKey, Rc<dyn Fn(&WsRecvResult<ServerMsgType>) -> bool>>;

pub type WsCallbackType<T> = StoredValue<Option<Rc<Closure<T>>>>;
pub type WsChannelsType<ServerMsg> = StoredValue<HashMap<WsRouteKey, WsChannelType<ServerMsg>>>;

#[derive(Clone, Debug)]
pub struct WsChannel<
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + crate::Send + Debug + 'static,
> {
    channel: WsChannelsType<ServerMsg>,
    pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    ws: StoredValue<Option<WebSocket>>,
    ws_url: StoredValue<Option<String>>,
    key: WsRouteKey,
    phantom: PhantomData<ClientMsg>,
    skip_if_awaiting_response: bool,
    timeout: Option<TimeDelta>,
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
    pub fn new(
        ws_url: StoredValue<Option<String>>,
        global_msgs_closures: WsChannelsType<ServerMsg>,
        ws: StoredValue<Option<WebSocket>>,
        global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
        skip_if_awaiting_response: bool,
        timeout: Option<TimeDelta>,
    ) -> Self {
        let channel_key: WsRouteKey = u128::generate_key();

        let create_channel = || {
            global_msgs_closures.update_value({
                move |channels| {
                    let Some(channel) = channels.get_mut(&channel_key) else {
                        let mut channel_callbacks: WsChannelCallbacksType<ServerMsg> =
                            HashMap::new();
                        let time = Some((
                            Utc::now(),
                            TimeDelta::microseconds(TIMEOUT_SECS * 1000 * 1000),
                        ));

                        let channel = WsChannelType::new(time, channel_callbacks);
                        channels.insert(channel_key, channel);
                        return;
                    };
                    warn!(
                        "ws({}): channel already exists: {}",
                        ws_url.get_value().unwrap_or("error".to_string()),
                        channel_key
                    );
                }
            })
        };

        #[cfg(target_arch = "wasm32")]
        {
            create_channel();
        }

        let detach = Owner::current().is_none();
        if !detach {
            on_cleanup({
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
            pending_client_msgs: global_pending_client_msgs,
            ws,
            ws_url,
            key: channel_key,
            phantom: PhantomData,
            skip_if_awaiting_response,
            timeout,
        }
    }

    #[track_caller]
    pub fn recv(&self, on_receive: impl Fn(&WsRecvResult<ServerMsg>) -> bool + 'static) {
        let channel_key = self.key;
        let callback_key = crate::location_hash();
        self.ws.with_value(|ws| -> Result<(), WsError> {
            self.channel.update_value({
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
                    let waiting_for_response = self.channel.with_value(|channels| {
                        channels
                            .get(&channel_key)
                            .map(|channel| channel.waiting_for_response)
                            .unwrap_or(false)
                    });
                    if waiting_for_response {
                        return Ok(WsResourcSendResult::Skipped);
                    }
                }
                let package: (WsRouteKey, ClientMsg) = (channel_key, client_msg);

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

                self.pending_client_msgs
                    .update_value(|pending| pending.push(bytes));

                self.set_is_waiting_for_response();
                Ok(WsResourcSendResult::Queued)
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

    fn remove_callback(&self, callback_key: WsRouteKey) -> bool {
        let channel_key = self.key;
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
