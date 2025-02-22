use std::marker::PhantomData;
use std::{collections::HashMap, fmt::Debug, rc::Rc};

use chrono::{DateTime, TimeDelta, Utc};
use leptos::*;
use leptos_use::use_window;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, trace, warn};
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen};
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

// pub mod channel;
// pub mod channel_builder;
// pub mod runtime;

pub const TIMEOUT_SECS: i64 = 30;

// #[derive(Clone, Copy, Debug)]
// pub enum WsOnRecvAction {
//     RemoveCallback,
//     RemoveTimeout,
//     None,
// }

pub type WsRouteKey = u128;
pub type WsPackage<MsgType: Clone + 'static> = (WsRouteKey, MsgType);

pub type RecvCallbacksType<ServerMsgType> = HashMap<u128, Rc<dyn Fn(&ServerMsgType, &mut bool)>>;
pub type TimeoutCallbacksType = HashMap<u128, Rc<dyn Fn(&mut bool)>>;

// pub type WsChannelCallbacksType<ServerMsgType> =
// HashMap<u128, Rc<dyn Fn(&WsRecvResult<ServerMsgType>, &mut bool) >>;
pub type WsCallbackEventType<T> = StoredValue<Option<Rc<Closure<T>>>>;
pub type WsChannelsType<ServerMsg> = StoredValue<HashMap<u128, WsChannel<ServerMsg>>>;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
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

#[derive(Clone)]
pub struct WsRuntime<ServerMsg: Clone + Receive + 'static, ClientMsg: Clone + Send + 'static> {
    pub ws: StoredValue<Ws<ServerMsg>>,
    pub instance: StoredValue<Option<Instance>>,
    pub bindings: StoredValue<Option<Bindings>>,
    //pub cleanup: StoredValue<Box<dyn Fn()>>,
    pub phantom: PhantomData<ClientMsg>,
    //pub connected: RwSignal<bool>,
}

#[derive(Clone)]
pub struct Ws<ServerMsg: Clone + Receive + 'static> {
    pub channels: HashMap<u128, WsChannel<ServerMsg>>,
    // pub global_on_open_callbacks: HashMap<WsRouteKey, Rc<dyn Fn()>>,
    // pub global_on_close_callbacks: HashMap<WsRouteKey, Rc<dyn Fn()>>,
    pub global_on_connect_callbacks: HashMap<WsRouteKey, Rc<dyn Fn(bool)>>,
    pub pending_client_msgs: Vec<Vec<u8>>,
}

pub struct Bindings {
    pub ws_on_msg: Closure<dyn FnMut(MessageEvent)>,
    pub ws_on_err: Closure<dyn FnMut(ErrorEvent)>,
    pub ws_on_open: Closure<dyn FnMut()>,
    pub ws_on_close: Closure<dyn FnMut()>,
}

pub struct Instance {
    pub ws: WebSocket,
    pub ws_url: String,
    
}

#[derive(Clone)]
pub struct WsChannel<ServerMsg: Clone + Receive + 'static> {
    pub waiting_for_cleanup: bool,
    pub waiting_for_response: u32,
    pub timeout_duratoin: Option<TimeDelta>,
    pub timeout_since: Option<(DateTime<chrono::Utc>)>,
    pub recv_callbacks: RecvCallbacksType<ServerMsg>,
    pub timeout_callbacks: TimeoutCallbacksType,
}

// #[derive(Clone, Debug)]
// pub struct WsChannelHandle<
//     ServerMsg: Clone + Receive + Debug + 'static,
//     ClientMsg: Clone + crate::Send + Debug + 'static,
// > {
//     channels: WsChannelsType<ServerMsg>,
//     global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
//     ws: StoredValue<Option<WebSocket>>,
//     ws_url: StoredValue<Option<String>>,
//     key: u128,
//     phantom: PhantomData<ClientMsg>,
//     //single_fire: bool,
//     timeout: Option<TimeDelta>,
//     is_connected: RwSignal<bool>,
// }

#[derive(Clone)]
pub struct WsChannelHandle<
    ServerMsg: Clone + Receive + 'static,
    ClientMsg: Clone + crate::Send + 'static,
> {
    runtime: WsRuntime<ServerMsg, ClientMsg>,
    key: u128,
}

impl<ServerMsg: Clone + Receive + Debug + 'static, ClientMsg: Clone + Send + Debug + 'static> Copy
    for WsRuntime<ServerMsg, ClientMsg>
{
}

// pub trait KeyGen
// where
//     Self: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
// {
//     fn generate_key() -> Self;
// }

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

// impl Default for WasmCallbacks {
//     fn default() -> Self {
//         Self {
//             ws_on_msg: None,
//             ws_on_err: None,
//             ws_on_open: None,
//             ws_on_close: None,
//         }
//     }
// }

impl<ServerMsg: Clone + Receive + Debug + 'static> Default for Ws<ServerMsg> {
    fn default() -> Self {
        Self {
            channels: HashMap::new(),
            // global_on_open_callbacks: HashMap::new(),
            // global_on_close_callbacks: HashMap::new(),
            global_on_connect_callbacks: HashMap::new(),
            pending_client_msgs: Vec::new(),
            // ws: None,
            // ws_url: None,
        }
    }
}

impl<ServerMsg: Clone + Receive + Debug + 'static, ClientMsg: Clone + Send + Debug + 'static>
    Default for WsRuntime<ServerMsg, ClientMsg>
{
    fn default() -> Self {

        let ws = StoredValue::new(Ws::default());
     
        #[cfg(not(target_arch = "wasm32"))]
        {
            return Self {
                bindings: StoredValue::new(None),
                instance: StoredValue::new(None),
                ws,
                phantom: PhantomData,
            };
        }

        
        let instance = StoredValue::new(None);

        let ws_on_msg = {
            let ws = ws.clone();
            let instance = instance.clone();

            Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                span_ws(instance);
                let _span = tracing::span!(tracing::Level::TRACE, "RUNTIME-RECV");

                //Self::ws_on_msg(channels, e);
            })
        };

        let ws_on_err = {
            let ws = ws.clone();
            let instance = StoredValue::new(None);

            Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
                span_ws(instance);
                let _span = tracing::span!(tracing::Level::TRACE, "RUNTIME-ERR");
                Self::ws_on_err(e);
            })
        };

        let ws_on_open = {
            let ws = ws.clone();
            let instance = StoredValue::new(None);

            Closure::<dyn FnMut()>::new(move || {
                span_ws(instance);
                let _span = tracing::span!(tracing::Level::TRACE, "RUNTIME-OPEN");
                //Self::ws_on_open(ws, ws_connected, ws_pending, ws_on_ws_state_closures);
            })
        };

        let ws_on_close = {
            let ws = ws.clone();
            let instance = StoredValue::new(None);

            Closure::<dyn FnMut()>::new(move || {
                span_ws(instance);
                let _span = tracing::span!(tracing::Level::TRACE, "RUNTIME-CLOSE");
                //Self::ws_on_close(ws, ws_connected, ws_on_ws_state_closures);
            })
        };

        let bindings = StoredValue::new(Some(Bindings {
            ws_on_open,
            ws_on_close,
            ws_on_err,
            ws_on_msg,
        }));

     

        let _reconnect_interval = leptos_use::use_interval_fn(
            {
                move || {
                    span_ws(instance);

                    instance.update_value(|instance| {
                        let Some(ref mut instance) = instance else {
                            return;
                        };

                        // bindings.with_value(|bindings| {
                        //     let Some(ref bindings) = bindings else {
                        //         return;
                        //     };


                        //     let result = instance.connect(bindings);
                        //     if let Err(err) = result {
                        //         tracing::error!("{err}");
                        //     }
                        // })

                        let result = instance.connect(bindings);
                        if let Err(err) = result {
                            tracing::error!("{err}");
                        }
                        

                        //let url = &instance.ws_url;

                    });
                }
            },
            1000,
        );
        
        let _timeout_interval = leptos_use::use_interval_fn(
            {
                move || {
                    span_ws(instance);

                    // let Some(instance) = instance else {
                    //     tracing::error!("error");
                    //     return;
                    // };

                    let _span = tracing::span!(tracing::Level::TRACE, "HANDLE-TIMEOUT");

                    let is_closed = instance.with_value(move |instance| {
                        instance
                            .as_ref()
                            .and_then(|instance| {
                                Some(instance.ws.ready_state() != WebSocket::OPEN)
                            })
                            .unwrap_or(false)
                    });

                    if is_closed {
                        //trace!("skipped, ws closed.",);
                        return;
                    }

                    let time = Utc::now();

                    let channels: Vec<(WsRouteKey, TimeoutCallbacksType)> = ws
                        .try_update_value(|ws| {
                            let mut output: Vec<(WsRouteKey, TimeoutCallbacksType)> =
                                Vec::new();

                            for (i, (channel_key, channel)) in
                                ws.channels.iter_mut().enumerate()
                            {
                                let Some(delta) = channel.timeout_duratoin else {
                                    continue;
                                };

                                if channel.waiting_for_response == 0 {
                                    continue;
                                }

                                let Some(since) = channel.timeout_since else {
                                    channel.timeout_since = Some(time);
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
                                    time - since,
                                    delta,
                                    channel.waiting_for_response
                                );

                                if time - since > delta {
                                    trace!("found callback: {:?}", channel_key);
                                    output.push((
                                        channel_key.clone(),
                                        channel.timeout_callbacks.clone(),
                                    ));
                                }
                            }

                            output
                        })
                        .unwrap_or_default();

                    for (channel_key, callbacks) in channels {
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

                            let mut remove_callback = false;
                            callback(&mut remove_callback);

                            if remove_callback {
                                Self::remove_callback(
                                    ws,
                                    channel_key,
                                    callback_key,
                                );
                            }
                        }

                        Self::update_timout(ws, channel_key);
                    }
                }
            },
            1000,
        );

        let cleanup_intervals = move || {
            (_reconnect_interval.pause)();
            (_timeout_interval.pause)();
        };

        //let cleanup = StoredValue::new(Box::new(cleanup));

        // let create_bindings = {
        //     move || -> Bindings {
        //         trace!("connecting...");
        //         let ws = WebSocket::new(&url).unwrap();

        //         ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        //         ws.set_onmessage(Some((**ws_on_msg).as_ref().unchecked_ref()));
        //         ws.set_onerror(Some((**ws_on_err).as_ref().unchecked_ref()));
        //         ws.set_onopen(Some((**ws_on_open).as_ref().unchecked_ref()));
        //         ws.set_onclose(Some((**ws_on_close).as_ref().unchecked_ref()));

        //         ws
        //     }
        //  };
        // fn what() {

        // }

        // let a = StoredValue::new(Box::new(what));
        // let b = StoredValue::new(Box::new(cleanup));

        on_cleanup(move || {
            cleanup_intervals();
        });
        
        Self {
            ws,
            instance,
            bindings,
            //cleanup:  StoredValue::new(Box::new(what)),
            phantom: PhantomData,
            //connected: RwSignal::new(false),
        }
    }
}

// impl KeyGen for u128 {
//     fn generate_key() -> Self {
//         uuid::Uuid::new_v4().as_u128()
//     }
// }

// impl KeyGen for String {
//     fn generate_key() -> Self {
//         uuid::Uuid::new_v4().to_string()
//     }
// }

impl<ServerMsg: Clone + Receive + Debug + 'static, ClientMsg: Clone + Send + Debug + 'static>
    WsRuntime<ServerMsg, ClientMsg>
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn channel(&self) -> WsChannelHandle<ServerMsg, ClientMsg> {
        span_ws(self.instance);

        let channel_key = ws_rand_key();

        WsChannelHandle {
            key: channel_key,
            runtime: *self,
        }

        // ChannelBuilder::new(
        //     self.ws_url,
        //     self.channels,
        //     self.ws,
        //     self.pending_client_msgs,
        // //    self.connected,
        // )
    }

    pub fn close(&self) {
        span_ws(self.instance);

        self.instance.update_value(|instance| {
            let Some(instance) = instance.take() else {
                trace!("already closed");
                return;
            };

            instance.close();
        });
    }

    pub fn send(&self, data: &[u8]) {
        self.instance.with_value(|instance| {
            let Some(instance) = instance else {
                return;
            };
            if instance.ws.ready_state() == WebSocket::OPEN {
                instance.ws.send_with_u8_array(data);
            } else {
                self.ws.update_value(|ws| {
                    ws.pending_client_msgs.push(data.to_vec());
                });
            }
        });
    }

    // pub fn add_callback(&self, id: u128, callback: impl Fn(&ServerMsg, &mut bool) + 'static) {
    //     self.channels.update_value(|channels| {
    //         let channel = channels.entry(id).or_insert_with(|| {
    //             WsChannelType::new(None, HashMap::new())
    //         });
    //         channel.callbacks.insert(id, Rc::new(callback));
    //     });
    // }

    // pub fn remove_callback(&self, id: u128, callback: impl Fn(&ServerMsg, &mut bool) + 'static) {
    //     self.channels.update_value(|channels| {
    //         let channel = channels.entry(id).or_insert_with(|| {
    //             WsChannelType::new(None, HashMap::new())
    //         });
    //         channel.callbacks.insert(id, Rc::new(callback));
    //     });
    // }

    #[track_caller]
    pub fn on_connect(&mut self, callback: impl Fn(bool) + 'static) {
        let on_connect_key = ws_rand_key();

        span_ws(self.instance);

        let _span = tracing::span!(
            tracing::Level::TRACE,
            "",
            "{}",
            format!("STATE_CALLBACK({})", on_connect_key)
        );

        let is_connected = self.is_connected();

        callback(is_connected);

        self.on_connect_add(on_connect_key, callback);

        self.on_connect_cleanup(on_connect_key);
    }

    pub fn is_connected(&self) -> bool {
        self.instance.with_value(|instance| {
            instance
                .as_ref()
                .map(|instance| instance.ws.ready_state() == WebSocket::OPEN)
                .unwrap_or(false)
        })
    }

    pub fn on_connect_add(&self, key: u128, callback: impl Fn(bool) + 'static) {
        trace!("added on_connect callback");
        self.ws.update_value(|v| {
            v.global_on_connect_callbacks.insert(key, Rc::new(callback));
        });
    }

    pub fn on_connect_cleanup(&self, key: u128) {
        on_cleanup({
            let ws = self.ws;
            let instance = self.instance;
            move || {
                span_ws(instance);

                let _span = tracing::span!(tracing::Level::TRACE, "STATE-CLEANUP");

                let _span = tracing::span!(
                    tracing::Level::TRACE,
                    "",
                    "{}",
                    format!("STATE_CALLBACK({})", key)
                );

                trace!("removed on_connect");
                ws.update_value(|ws| {
                    ws.global_on_connect_callbacks.remove(&key);
                });
            }
        });
    }

    pub fn get_url(&self) -> Option<String> {
        self.instance
            .with_value(move |instance| instance.as_ref().map(|v| v.ws_url.clone()))
    }

    pub fn get_url_str(&self) -> String {
        self.get_url().unwrap_or("disconnected".to_string())
    }

    pub fn connect(&self, port: u32) -> Result<(), ConnectError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return Ok(());
        }

        let path = get_ws_url(port)?;
        self.connect_to(&path);
        Ok(())
    }

    pub fn connect_to(&self, url: &str) -> Result<(), WsCreateErr> {
        let connect = || -> Result<(), WsCreateErr> {
            span_ws_str(url);

            let instance = self.instance;
            let bindings = self.bindings;
            let ws = self.ws;

            let _span = tracing::span!(tracing::Level::TRACE, "RUNTIME");

            // // let mut ws_on_msg = self.ws_on_msg;
            // // let mut ws_on_err = self.ws_on_err;
            // // let mut ws_on_open = self.ws_on_open;
            // // let mut ws_on_close = self.ws_on_close;
            // let channels = self.channels;
            // //let ws_connected = self.connected;
            // let ws_on_open_closures = self.global_on_open_callbacks;
            // let ws_on_close_closures = self.global_on_close_callbacks;
            // let ws_on_ws_state_closures = self.global_on_ws_state_change_callbacks;
            // let ws_pending = self.pending_client_msgs;
            // let ws = self.ws;

            //let (ws_on_msg_tx, ws_on_msg_rx) = futures::channel::mpsc::channel::<ServerMsg>(100);

            //ws_on_msg_rx.next().await;
            // ws_on_msg_tx.send(item).await;

            //ws.set_value(Some(create_ws()));
            

            let result = instance.try_update_value(|instance| -> Result<(), WsCreateErr> {
                let Some(current_instance) = instance.take() else {
                    *instance = Some(Instance::new(bindings, url)?);
                    return Ok(());
                };
                current_instance.close();
                *instance = Some(Instance::new(bindings, url)?);
                Ok(())
            });
            if let Some(result) = result {
                result?;
            }

            Ok(())
        };
        #[cfg(target_arch = "wasm32")]
        {
            connect()?;
        }
        Ok(())
    }

    fn ws_on_open(
        ws: StoredValue<Ws<ServerMsg>>,
        instance: StoredValue<Option<Instance>>,
        //connected: RwSignal<bool>,
        //socket_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
        // global_on_ws_state_callbacks: StoredValue<HashMap<WsRouteKey, Rc<dyn Fn(bool)>>>,
    ) {
        // trace!(
        //     "connected, ws_on_closeclosures left {}",
        //     global_on_ws_state_callbacks.with_value(|c| c.len())
        // );
        //connected.set(true);

        // ws.update_value(|ws| {
        //     instance.with_value(|instance| {
        //         let Some(instance) = instance else {
        //             tracing::error!("no instance found");
        //             return;
        //         };
                
        //     });
        // });


        Self::event_run_on_connection_callbacks(ws, instance);
        Self::event_flush_pending_client_msgs(ws, instance);
    }

    fn ws_on_close(ws: StoredValue<Ws<ServerMsg>>, instance: StoredValue<Option<Instance>>) {
        info!("disconnected");

        Self::event_run_on_connection_callbacks(ws, instance);
    }

    fn ws_on_err(e: ErrorEvent) {
        error!("{:?}", e);
    }

    fn ws_on_msg(ws: StoredValue<Ws<ServerMsg>>, e: MessageEvent) {
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

        let (channel_key, server_msg) = match ServerMsg::recv_from_vec(&bytes) {
            Ok(v) => v,
            Err(err) => {
                error!("data decoding: {}", err);
                return;
            }
        };

        let _span = tracing::span!(
            tracing::Level::TRACE,
            "",
            "{}",
            format!("CHANNEL({:#01x})", channel_key)
        )
        .entered();

        trace!("data: \n{:#?}", &server_msg);

        Self::update_timout(ws, channel_key);
    }

    fn event_flush_pending_client_msgs(
        ws: StoredValue<Ws<ServerMsg>>, instance: StoredValue<Option<Instance>>
        //socket_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    ) {
        instance.with_value(|instance| {
            let Some(instance) = instance else {
                tracing::error!("no instance found");
                return;
            };
            ws.update_value(|ws| {
                let msgs = &mut ws.pending_client_msgs;
                trace!("sending from queue amount: {}", msgs.len());
                let mut index: usize = 0;
                for msg in msgs.iter() {
                    trace!("sending from queue {}: {:?}", index, msg);
                    let result = instance.ws.send_with_u8_array(msg);
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
        });
       
    }

    fn event_run_on_connection_callbacks(ws: StoredValue<Ws<ServerMsg>>, instance: StoredValue<Option<Instance>>) {
        let callbacks = ws.with_value(|ws| ws.global_on_connect_callbacks.clone());

        if !callbacks.is_empty() {
            let is_connected = is_connected(instance);

            for (key, callback) in callbacks {
                let _span = tracing::span!(
                    tracing::Level::TRACE,
                    "",
                    "{}",
                    format!("ON_CONNECTION({})", key)
                );

                trace!("running on_connection...");
                callback(is_connected);
            }
        }
    }

    fn event_run_callbacks(ws: StoredValue<Ws<ServerMsg>>, package: WsPackage<ServerMsg>) {
        let channel_key: WsRouteKey = package.0;
        let server_msg = package.1;
        //let server_msg = WsRecvResult::Ok(package.1);

        let channel: Option<WsChannel<ServerMsg>> = ws.with_value(move |ws| {
            let Some(channel) = ws.channels.get(&channel_key) else {
                error!("channel not found {:?}", &channel_key);
                return None;
            };

            Some(channel.clone())
        });

        let Some(channel) = channel else {
            return;
        };

        for (callback_key, callback) in channel.recv_callbacks {
            let _span = tracing::span!(
                tracing::Level::TRACE,
                "",
                "{}",
                format!("CALLBACK({})", callback_key)
            )
            .entered();
            trace!("running callback");

            let mut remove_callback = false;
            callback(&server_msg, &mut remove_callback);
            if remove_callback {
                Self::remove_callback(ws, channel_key, callback_key);
            }
            
        }
        Self::update_timout(ws, channel_key);
    }

    pub fn remove_callback(
        ws: StoredValue<Ws<ServerMsg>>,
        channel_key: WsRouteKey,
        callback_key: u128,
       // keep_open: bool,
    ) {
        ws.update_value(|ws| {
            let Some(channel) = ws.channels.get_mut(&channel_key) else {
                error!("channel not found",);
                return;
            };

            let result = channel.recv_callbacks.remove(&callback_key);
            if let Some(_) = result {
                trace!("removed callback");
            } else {
                error!("callback not found");
            }

           
        });
    }

    pub fn update_timout(ws: StoredValue<Ws<ServerMsg>>, channel_key: WsRouteKey) {
        ws.update_value(|ws| {
            let Some(channel) = ws.channels.get_mut(&channel_key) else {
                error!("channel not found");
                return;
            };

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
        });
    }
}

impl<
        'a,
        ServerMsg: Clone + Debug + Receive + serde::Deserialize<'a> + 'static,
        ClientMsg: Clone + Debug + crate::Send + serde::Serialize + 'static,
    > WsChannelHandle<ServerMsg, ClientMsg>
{
    pub fn recv(&self, callback: impl Fn(&ServerMsg, &mut bool) + 'static) {
        span_ws(self.runtime.instance);

        let channel_key = self.key;

        let has_inserted = self.runtime.ws.try_update_value(|ws| {
            let Some(channel) = ws.channels.get_mut(&channel_key) else {
                error!("channel not found");
                return false;
            };
            channel
                .recv_callbacks
                .insert(channel_key, Rc::new(callback));
            false
        });

        if has_inserted.unwrap_or(false) {
            on_cleanup({
                let ws = self.runtime.ws;
                let instance = self.runtime.instance;
                move || {
                    span_ws(instance);

                    let _span = tracing::span!(tracing::Level::TRACE, "RECV-CLEANUP");

                    let _span = tracing::span!(
                        tracing::Level::TRACE,
                        "",
                        "{}",
                        format!("CHANNEL_CALLBACK({})", channel_key)
                    );

                    ws.update_value(|ws| {
                        let Some(channel) = ws.channels.get_mut(&channel_key) else {
                            error!("channel not found");
                            return;
                        };
                        trace!("removed channel callback");
                        channel.recv_callbacks.remove(&channel_key);
                    });
                }
            });
        }
    }

    pub fn send(&self, msg: &ClientMsg) {
        span_ws(self.runtime.instance);

        let bytes = match bincode::serialize::<ClientMsg>(msg) {
            Ok(v) => v,
            Err(err) => {
                tracing::error!("{}", err);
                return;
            }
        };

        self.runtime.send(&bytes);
    }
}

impl Instance {
    pub fn new(
        bindings: StoredValue<Option<Bindings>>,
       // cleanup: impl Fn() + 'static,
        url: &str,
    ) -> Result<Self, WsCreateErr> {
        trace!("connecting to {url}...");

        Ok(Instance {
            ws: create_ws(bindings, url)?,
            ws_url: url.to_string(),
         //   cleanup: Box::new(cleanup),
        })
    }

    pub fn is_connected(&self) -> bool {
        self.ws.ready_state() == web_sys::WebSocket::OPEN
    }

    pub fn connect(
        &mut self,
        bindings: StoredValue<Option<Bindings>>,
    ) -> Result<(), WsCreateErr> {
        if self.ws.ready_state() == WebSocket::CLOSED {
            trace!("connecting...");
            self.ws = create_ws(bindings, &self.ws_url)?;
        }
        Ok(())
    }

    pub fn reconnect(
        &mut self,
        bindings: StoredValue<Option<Bindings>>,
    ) -> Result<(), WsCreateErr> {
        trace!("reconnecting...");
        self.close();
        self.ws = create_ws(bindings, &self.ws_url)?;
        Ok(())
    }

    pub fn close(&self) {
        let ready_state = self.ws.ready_state();
        match ready_state {
            web_sys::WebSocket::OPEN | web_sys::WebSocket::CONNECTING => {
                trace!("closing...");
                let result = self.ws.close();
                if let Err(_) = result {
                    error!("error closing");
                }
            }
            web_sys::WebSocket::CLOSING => {
                trace!("already closing...");
            }
            _ => {
                //trace!("already closed.");
            }
        }
    }
}

pub fn is_connected(instance: StoredValue<Option<Instance>>) -> bool {
    instance.with_value(|instance| instance.as_ref().map(|instance| instance.is_connected()).unwrap_or(false))
}

pub fn ws_rand_key() -> u128 {
    uuid::Uuid::new_v4().as_u128()
}

#[track_caller]
pub fn ws_loc_key() -> u128 {
    xxhash_rust::xxh3::xxh3_128(std::panic::Location::caller().to_string().as_bytes())
}

pub fn create_ws(
    bindings: StoredValue<Option<Bindings>>,
    url: &str,
) -> Result<web_sys::WebSocket, WsCreateErr> {
    let ws = WebSocket::new(&url).map_err(|err| WsCreateErr::FailedToCreateWs)?;

    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    bindings.with_value(|bindings| {
        let Some(bindings) = bindings else {
            return;
        };
        ws.set_onmessage(Some((bindings.ws_on_msg).as_ref().unchecked_ref()));
        ws.set_onerror(Some((bindings.ws_on_err).as_ref().unchecked_ref()));
        ws.set_onopen(Some((bindings.ws_on_open).as_ref().unchecked_ref()));
        ws.set_onclose(Some((bindings.ws_on_close).as_ref().unchecked_ref()));
    });

    Ok(ws)
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

#[track_caller]
pub fn span_ws(instance: StoredValue<Option<Instance>>) {
    span_ws_str(
        &instance
            .with_value(move |wasm_callbacks| wasm_callbacks.as_ref().map(|v| v.ws_url.clone()))
            .unwrap_or("disconnected".to_string()),
    )
}

#[track_caller]
pub fn span_ws_str(url: &str) {
    let _span = tracing::span!(tracing::Level::TRACE, "", "{}", format!("ws({})", url));
}

#[derive(Error, Debug)]
pub enum WsCreateErr {
    #[error("failed to create ws")]
    FailedToCreateWs,
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
