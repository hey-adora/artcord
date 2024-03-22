use std::marker::PhantomData;
use std::rc::Rc;
use std::{collections::HashMap, fmt::Debug};

use cfg_if::cfg_if;

use leptos::*;
use leptos_use::use_window;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, trace, warn};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

// const WS_TIMEOUT_MS: i64 = 30000;
// const WS_TIMEOUT_MS_MARGIN: i64 = 100;

// pub trait Send<T: KeyGen<T>> {
//     fn send() {
//         stuff.send(T::generate_key())
//     }
// }

// pub trait Recv<T: KeyGen<T>> {
//     fn recv() {
//         stuff.recv(T::generate_key())
//     }
// }

// pub trait KeyGen<T> {
//     fn generate_key() -> T;
// }

//trait KeyGenTraits = Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, Hash)]
pub enum WsRouteKey<
    TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
> {
    Perm(PermKeyType),
    Temp(TempKeyType),
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, Hash)]
pub struct WsPackage<
    TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    Data: Clone + 'static,
> {
    pub key: WsRouteKey<TempKeyType, PermKeyType>,
    pub data: Data,
}

// struct WsServerPackage<
//     KeyGen: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
//     PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
//     WsTempRoute: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
//     ServerMsg: Clone + Receive<WsRouteKind<KeyGen, PermKeyType, WsTempRoute>> + 'static,
// > {
//     key: WsRouteKind<KeyGen, PermKeyType, WsTempRoute>,
//     data: ServerMsg,
// }



enum WsMsg<C, S> {
    Client(C),
    Server(S),
}

enum WsResponse<T> {
    Ok(T),
    SendErr,
    TimeOut
}

enum ExampleRoutes {
    User(WsMsg<u128, String>),
}

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
    TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
>
{
    fn send_as_vec(package: &WsPackage<TempKeyType, PermKeyType, Self>) -> Result<Vec<u8>, String>
    where
        Self: Clone;
}

pub trait Receive<
    TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
>
{
    fn recv_from_vec(bytes: &[u8]) -> Result<WsPackage<TempKeyType, PermKeyType, Self>, String>
    where
        Self: std::marker::Sized + Clone;
}

type WsCallback<T> = StoredValue<Option<Rc<Closure<T>>>>;
type GlobalMsgCallbacks<
    TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
    ServerMsg: Clone + 'static,
> = StoredValue<
    HashMap<WsRouteKey<TempKeyType, PermKeyType>, HashMap<TempKeyType, Rc<dyn Fn(&ServerMsg)>>>,
>;

#[derive(Clone, Debug)]
pub struct WsRuntime<
    TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive<TempKeyType, PermKeyType> + Debug + 'static,
    ClientMsg: Clone + Send<TempKeyType, PermKeyType> + Debug + 'static,
> {
    pub global_msgs_callbacks: GlobalMsgCallbacks<TempKeyType, PermKeyType, ServerMsg>,
    pub global_on_open_callbacks: StoredValue<HashMap<TempKeyType, Rc<dyn Fn()>>>,
    pub global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    pub ws: StoredValue<Option<WebSocket>>,
    pub ws_url: StoredValue<Option<String>>,
    pub ws_on_msg: WsCallback<dyn FnMut(MessageEvent)>,
    pub ws_on_err: WsCallback<dyn FnMut(ErrorEvent)>,
    pub ws_on_open: WsCallback<dyn FnMut()>,
    pub ws_on_close: WsCallback<dyn FnMut()>,
    phantom: PhantomData<ClientMsg>,
}

impl<
        TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        ServerMsg: Clone + Receive<TempKeyType, PermKeyType> + Debug + 'static,
        ClientMsg: Clone + Send<TempKeyType, PermKeyType> + Debug + 'static,
    > Copy for WsRuntime<TempKeyType, PermKeyType, ServerMsg, ClientMsg>
{
}

impl<
        TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        ServerMsg: Clone + Receive<TempKeyType, PermKeyType> + Debug + 'static,
        ClientMsg: Clone + Send<TempKeyType, PermKeyType> + Debug + 'static,
    > Default for WsRuntime<TempKeyType, PermKeyType, ServerMsg, ClientMsg>
{
    fn default() -> Self {
        Self {
            global_msgs_callbacks: StoredValue::new(HashMap::new()),
            global_on_open_callbacks: StoredValue::new(HashMap::new()),
            global_pending_client_msgs: StoredValue::new(Vec::new()),
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
        TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
        PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
        ServerMsg: Clone + Receive<TempKeyType, PermKeyType> + Debug + 'static,
        ClientMsg: Clone + Send<TempKeyType, PermKeyType> + Debug + 'static,
    > WsRuntime<TempKeyType, PermKeyType, ServerMsg, ClientMsg>
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
        #[cfg(target_arch = "wasm32")]
        {
            let url = String::from(url);

            let ws_on_msg = self.ws_on_msg;
            let ws_on_err = self.ws_on_err;
            let ws_on_open = self.ws_on_open;
            let ws_on_close = self.ws_on_close;
            let ws_closures = self.global_msgs_callbacks;
            let ws_on_open_closures = self.global_on_open_callbacks;
            let ws_pending = self.global_pending_client_msgs;
            let ws = self.ws;

            ws_on_msg.set_value({
                let url = url.clone();
                Some(Rc::new(Closure::<dyn FnMut(_)>::new(
                    move |e: MessageEvent| Self::ws_on_msg(&url, ws_closures, e),
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
                Some(Rc::new(Closure::<dyn FnMut()>::new(move || {
                    Self::ws_on_open(&url, ws_pending, ws_on_open_closures, ws)
                })))
            });

            ws_on_close.set_value({
                let url = url.clone();
                Some(Rc::new(Closure::<dyn FnMut()>::new(move || {
                    Self::ws_on_close(&url, ws_on_open_closures)
                })))
            });

            let create_ws = {
                let url = url.clone();
                move || -> WebSocket {
                    info!("ws({}): connecting", &url);
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
                            info!("ws: reconnecting {}", url);
                            ws.set_value(Some(create_ws()));
                        }
                    }
                },
                1000,
            );
        }
    }

    pub fn create_singleton(&self) -> WsSingleton<TempKeyType, PermKeyType, ServerMsg, ClientMsg> {
        // let global_state = use_context::<LeptosWebSockets<WsRoute, ServerMsg, ClientMsg>>()
        //     .expect("Failed to provide global state");
        // let ws_closures = global_state.global_msgs_closures;
        // let ws = global_state.ws;
        // let socket_pending_client_msgs = global_state.global_pending_client_msgs;
        WsSingleton::<TempKeyType, PermKeyType, ServerMsg, ClientMsg>::new(
            self.ws_url,
            self.global_msgs_callbacks,
            self.ws,
            self.global_pending_client_msgs,
        )
    }

    //fn generate_key() -> WsRoute;

    fn ws_on_open(
        url: &str,
        socket_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
        global_on_open_callbacks: StoredValue<HashMap<TempKeyType, Rc<dyn Fn()>>>,
        ws: StoredValue<Option<WebSocket>>,
    ) {
        info!("ws({}): connected", url);

        Self::run_on_open_callbacks(url, global_on_open_callbacks);
        Self::flush_pending_client_msgs(url, socket_pending_client_msgs, global_on_open_callbacks, ws);
        
    }

    fn ws_on_close(url: &str, global_on_open_callbacks: StoredValue<HashMap<TempKeyType, Rc<dyn Fn()>>>,) {
        //let keys = global_on_open_callbacks.with_value(|callbacks| callbacks.keys().cloned().collect::<Vec<TempKeyType>>() );
        trace!("ws({}): disconnect: on-open closures left: {}", url, global_on_open_callbacks.with_value(|c| c.len()));
        info!("ws({}): disconnected", url);
    }

    fn ws_on_err(url: &str, e: ErrorEvent) {
        error!("WS({}): error: {:?}", url, e);
    }

    fn ws_on_msg(
        url: &str,
        closures: GlobalMsgCallbacks<TempKeyType, PermKeyType, ServerMsg>,
        e: MessageEvent,
    ) {
        let data = e.data().dyn_into::<js_sys::ArrayBuffer>();
        let Ok(data) = data else {
            return;
        };
        let array = js_sys::Uint8Array::new(&data);
        let bytes: Vec<u8> = array.to_vec();

        if bytes.is_empty() {
            trace!("ws({}): recv empty msg.", url);
            return;
        };

        let server_msg = ServerMsg::recv_from_vec(&bytes);
        let Ok(server_msg) = server_msg else {
            error!(
                "ws({}): error decoding msg: {}",
                url,
                server_msg.err().unwrap()
            );
            return;
        };

        Self::execute(url, closures, server_msg);
    }

    fn flush_pending_client_msgs(
        url: &str,
        socket_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
        global_on_open_callbacks: StoredValue<HashMap<TempKeyType, Rc<dyn Fn()>>>,
        ws: StoredValue<Option<WebSocket>>,
    ) {
        ws.with_value(|ws| {
            if let Some(ws) = ws {
                socket_pending_client_msgs.update_value(|msgs| {
                    trace!("ws({}): sending msgs from queue, left: {}", url, msgs.len());
                    let mut index: usize = 0;
                    for msg in msgs.iter() {
                        let result = ws.send_with_u8_array(msg);
                        if result.is_err() {
                            warn!("ws({}): failed to send msg {}:{:?}", url, index, msg);
                            break;
                        }
                        trace!("ws({}): sent msg {} from queue: {:?}", url, index, msg);
                        index += 1;
                    }
                    if index < msgs.len() && index > 0 {
                        *msgs = (&msgs[index..]).to_vec();
                        trace!("ws({}): msg left in queue: {}", url, msgs.len());
                    }
                    //debug!("ws: after-queue: on_open callbacks, left: {:#?}", global_on_open_callbacks);
                });
            } else {
                warn!("ws({}): not initialized.", url);
            }
        });
    }

    fn run_on_open_callbacks(
        url: &str,
        global_on_open_callbacks: StoredValue<HashMap<TempKeyType, Rc<dyn Fn()>>>,
     //   ws: StoredValue<Option<WebSocket>>,
    ) {
        let callbacks = global_on_open_callbacks.get_value();
        trace!("ws({}): running on_open callbacks... left: {:#?}", url, callbacks.len());
        for callback in callbacks.values() {
            callback();
        }
        // global_on_open_callbacks.update_value(|callbacks| {
            
        //     //debug!("ws: running on_open callbacks... left: {}", callbacks.len());
        //     trace!("ws: running on_open callbacks... left: {:#?}", callbacks.keys().cloned());
           

        //     // trace!("sending msgs from queue, left: {}", msgs.len());
        //     // let mut index: usize = 0;
        //     // for msg in msgs.iter() {
        //     //     let result = ws.send_with_u8_array(msg);
        //     //     if result.is_err() {
        //     //         warn!("failed to send msg {}:{:?}", index, msg);
        //     //         break;
        //     //     }
        //     //     trace!("sent msg {} from queue: {:?}", index, msg);
        //     //     index += 1;
        //     // }
        //     // if index < msgs.len() && index > 0 {
        //     //     *msgs = (&msgs[index..]).to_vec();
        //     //     trace!("msg left in queue: {}", msgs.len());
        //     // }
        // });

        // ws.with_value(|ws| {
        //     if let Some(ws) = ws {
        //         //let callbacks = self.global_on_open_callbacks.get_value();

        //     } else {
        //         warn!("ws: not initialized.");
        //     }
        // });
    }

    fn execute(
        url: &str,
        callbacks: GlobalMsgCallbacks<TempKeyType, PermKeyType, ServerMsg>,
        package: WsPackage<TempKeyType, PermKeyType, ServerMsg>,
    ) {
        let key: WsRouteKey<TempKeyType, PermKeyType> = package.key.clone();

        let callback_cluster: Option<HashMap<TempKeyType, Rc<dyn Fn(&ServerMsg)>>> = callbacks
            .with_value({
                let key = key.clone();
                move |socket_closures| {
                    trace!("ws({}): current callbacks: {:#?}", url, &socket_closures.keys());

                    let Some(f) = socket_closures.get(&key) else {
                        warn!("ws({}): Fn not found for {:?}", url, &key);
                        return None;
                    };

                    Some(f.clone())
                }
            });

        let Some(callback_cluster) = callback_cluster else {
            return;
        };

        match &key {
            WsRouteKey::Perm(_) => {
                for (_, callback) in callback_cluster {
                    callback(&package.data);
                }
            }
            WsRouteKey::Temp(_) => {
                for temp_key in callback_cluster.keys() {
                    let callback = callback_cluster.get(temp_key);
                    let Some(callback) = callback else {
                        warn!("ws({}): temp callback missing {:?}", url, temp_key);
                        continue;
                    };
                    callback(&package.data);
                }
                callbacks.update_value(|callbacks| {
                    callbacks.remove(&key);
                });
            }
        }

        //

        // socket_closures.remove(&key);
    }

    pub fn send(
        &self,
        perm_key: PermKeyType,
        client_msg: ClientMsg,
    ) -> Result<SendResult, SendError> {
 
        self.ws.with_value(|ws| -> Result<SendResult, SendError> {
            // let exists: Option<bool> = self.global_msgs_callbacks.try_update_value({
            //     move |global_msgs_closures| {
            //         let new_msg_closure = Rc::new(move |server_msg: &ServerMsg| {
            //             on_receive(server_msg);
            //         });

            //         let current_callback_cluster = global_msgs_closures.get(&perm_key);
            //         match current_callback_cluster {
            //             Some(current_callback_cluster) => {
            //                 //current_callback_cluster.insert(temp_key, new_msg_closure);
            //                 return true;
            //             }
            //             None => {
            //                 let mut callback_cluster: HashMap<TempKeyType, Rc<dyn Fn(&ServerMsg)>> = HashMap::new();
            //                 callback_cluster.insert(TempKeyType::generate_key(), new_msg_closure);
            //                 global_msgs_closures.insert(self.key.clone(), callback_cluster);
            //             }
            //         }
            //         // let Some(callback_cluster) = callback_cluster else {
            //         //     trace!("ws: ");
            //         //     return;
            //         // };
            //         //global_msgs_closures.insert(self.key.clone(), new_msg_closure);
            //         false
            //     }
            // });

            // if exists.unwrap_or(false) {
            //     return Ok(SendResult::Skipped);
            // }

            let package = WsPackage::<TempKeyType, PermKeyType, ClientMsg> {
                data: client_msg,
                key: WsRouteKey::Perm(perm_key),
            };
            let bytes = ClientMsg::send_as_vec(&package).map_err(SendError::Serializatoin)?;

            //let mut queue = true;

            if let Some(ws) = ws {
                let is_open = self.ws.with_value(move |ws| {
                    ws.as_ref()
                        .map(|ws| ws.ready_state() == WebSocket::OPEN)
                        .unwrap_or(false)
                });

                if is_open {
                    trace!("ws({}): msg \"{:?}\" sent", self.ws_url.get_value().unwrap_or("error".to_string()), &package);

                    return ws
                        .send_with_u8_array(&bytes)
                        .map(|_| {
                            let keys = self.global_on_open_callbacks.with_value(|callbacks| callbacks.keys().cloned().collect::<Vec<TempKeyType>>() );
                            //debug!("ws({}): after-send: on_open callbacks, left: {:#?}", self.ws_url.get_value().unwrap_or("error".to_string()), keys);
                            SendResult::Sent
                        })
                        .map_err(|e| {
                            // self.global_msgs_callbacks.update_value({
                            //     move |socket_closures| {
                            //         socket_closures.remove(&client_msg);
                            //     }
                            // });
                            SendError::SendError(
                                e.as_string()
                                    .unwrap_or(String::from("Failed to send web-socket package")),
                            )
                        });
                }
            }

            trace!("ws({}): msg \"{:?}\" pushed to queue", self.ws_url.get_value().unwrap_or("error".to_string()), &package);
            self.global_pending_client_msgs
                .update_value(|pending| pending.push(bytes));
            Ok(SendResult::Queued)
        })
    }

    pub fn on_open(&self, callback: impl Fn() + 'static) {
        let temp_key = TempKeyType::generate_key();

        self.global_on_open_callbacks.update_value({
            let temp_key = temp_key.clone();
            move |callbacks| {
                trace!("ws({}): adding on_open callback: {:#?}", self.ws_url.get_value().unwrap_or("error".to_string()), temp_key);
                callbacks.insert(temp_key, Rc::new(callback));
            }
        });

        on_cleanup({
            let callbacks = self.global_on_open_callbacks;
            let ws_url = self.ws_url;
            move || {
                callbacks.update_value({
                    move |callbacks| {
                        trace!("ws({}): cleanup: removing on_open callback: {:#?}", ws_url.get_value().unwrap_or("error".to_string()), temp_key);
                        callbacks.remove(&temp_key);
                    }
                });
            }
        });
    }

    pub fn on(&self, perm_key: PermKeyType, on_receive: impl Fn(&ServerMsg) + 'static) {
        let perm_key = WsRouteKey::<TempKeyType, PermKeyType>::Perm(perm_key);
        //let temp_key = WsRouteKey::<TempKeyType, PermKeyType>::Temp(TempKeyType::generate_key());
        let temp_key = TempKeyType::generate_key();
        self.global_msgs_callbacks.update_value({
            let temp_key = temp_key.clone();
            let perm_key = perm_key.clone();
            move |global_msgs_callbacks| {
                let new_msg_closure = Rc::new(move |server_msg: &ServerMsg| {
                    on_receive(server_msg);
                });

                //let key = WsRouteKey::Perm(perm_route);
                let current_callback_cluster: Option<
                    &mut HashMap<TempKeyType, Rc<dyn Fn(&ServerMsg)>>,
                > = global_msgs_callbacks.get_mut(&perm_key);
                match current_callback_cluster {
                    Some(current_callback_cluster) => {
                        current_callback_cluster.insert(temp_key, new_msg_closure);
                    }
                    None => {
                        let mut callback_cluster: HashMap<TempKeyType, Rc<dyn Fn(&ServerMsg)>> =
                            HashMap::new();
                        callback_cluster.insert(temp_key, new_msg_closure);
                        global_msgs_callbacks.insert(perm_key, callback_cluster);
                    }
                }

                //global_msgs_callbacks.insert(ws_route_key, new_msg_closure);
            }
        });
        on_cleanup({
            // let temp_key = temp_key.clone();
            // let perm_key = perm_key.clone();
            let callbacks = self.global_msgs_callbacks;
            let ws_url = self.ws_url.clone();
            move || {
                callbacks.update_value({
                    move |callbacks| {
                        //callbacks.get_mut(&perm_key).inspect(|callback_cluster| {callback_cluster.remove(&terp_key);}).map_or_else(|| trace!("ws: singletone cleanup: closure not found: {:?}", &perm_key));
                        let callback_cluster = callbacks.get_mut(&perm_key);
                        let Some(callback_cluster) = callback_cluster else {
                            trace!("ws({}): singletone cleanup: closure not found: {:?}", ws_url.get_value().unwrap_or("error".to_string()), &perm_key);
                            return;
                        };
                        callback_cluster.remove(&temp_key);
                    }
                });
            }
        });
    }
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

#[derive(Clone, Debug)]
pub struct WsSingleton<
    TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive<TempKeyType, PermKeyType> + Debug + 'static,
    ClientMsg: Clone + Send<TempKeyType, PermKeyType> + Debug + 'static,
> {
    global_msgs_closures: GlobalMsgCallbacks<TempKeyType, PermKeyType, ServerMsg>,
    global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    //socket_send_fn: StoredValue<Rc<dyn Fn(Vec<u8>)>>,
    ws: StoredValue<Option<WebSocket>>,
    ws_url: StoredValue<Option<String>>,
    key: WsRouteKey<TempKeyType, PermKeyType>,
    phantom: PhantomData<ClientMsg>,
}

impl<
        TempKeyType: KeyGen + Default + Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        PermKeyType: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug,
        ServerMsg: Clone + Receive<TempKeyType, PermKeyType> + Debug + 'static,
        ClientMsg: Clone + Send<TempKeyType, PermKeyType> + Debug + 'static,
    > WsSingleton<TempKeyType, PermKeyType, ServerMsg, ClientMsg>
{
    pub fn new(
        ws_url: StoredValue<Option<String>>,
        global_msgs_closures: GlobalMsgCallbacks<TempKeyType, PermKeyType, ServerMsg>,
        ws: StoredValue<Option<WebSocket>>,
        global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    ) -> Self {
        let ws_round_kind =
            WsRouteKey::<TempKeyType, PermKeyType>::Temp(TempKeyType::generate_key());
        on_cleanup({
            let key = ws_round_kind.clone();
            move || {
                global_msgs_closures.update_value({
                    move |socket_closures| {
                        socket_closures.remove(&key);
                    }
                });
            }
        });

        Self {
            global_msgs_closures,
            global_pending_client_msgs,
            ws,
            ws_url,
            key: ws_round_kind,
            phantom: PhantomData,
        }
    }

    pub fn send_once(
        &self,
        client_msg: ClientMsg,
        on_receive: impl Fn(&ServerMsg) + 'static,
    ) -> Result<TempSendResult, SendError> {
        self.ws
            .with_value(|ws| -> Result<TempSendResult, SendError> {
                //let ws = ws.as_ref().ok_or(SendError::WsNotInitialized)?;

                // let exists = self
                //     .global_msgs_closures
                //     .with_value(|socket_closures| socket_closures.contains_key(&self.key));
                let exists: Option<bool> = self.global_msgs_closures.try_update_value({
                    move |global_msgs_closures| {
                        let new_msg_closure = Rc::new(move |server_msg: &ServerMsg| {
                            on_receive(server_msg);
                        });

                        let current_callback_cluster = global_msgs_closures.get(&self.key);
                        match current_callback_cluster {
                            Some(current_callback_cluster) => {
                                //current_callback_cluster.insert(temp_key, new_msg_closure);
                                return true;
                            }
                            None => {
                                let mut callback_cluster: HashMap<
                                    TempKeyType,
                                    Rc<dyn Fn(&ServerMsg)>,
                                > = HashMap::new();
                                callback_cluster
                                    .insert(TempKeyType::generate_key(), new_msg_closure);
                                global_msgs_closures.insert(self.key.clone(), callback_cluster);
                            }
                        }
                        // let Some(callback_cluster) = callback_cluster else {
                        //     trace!("ws: ");
                        //     return;
                        // };
                        //global_msgs_closures.insert(self.key.clone(), new_msg_closure);
                        false
                    }
                });

                if exists.unwrap_or(false) {
                    return Ok(TempSendResult::Skipped);
                }

                let package = WsPackage::<TempKeyType, PermKeyType, ClientMsg> {
                    data: client_msg,
                    key: self.key.clone(),
                };
                let bytes = ClientMsg::send_as_vec(&package).map_err(SendError::Serializatoin)?;

                //let mut queue = true;

                if let Some(ws) = ws {
                    let is_open = self.ws.with_value(move |ws| {
                        ws.as_ref()
                            .map(|ws| ws.ready_state() == WebSocket::OPEN)
                            .unwrap_or(false)
                    });

                    if is_open {
                        return ws
                            .send_with_u8_array(&bytes)
                            .map(|_| TempSendResult::Sent)
                            .map_err(|e| {
                                self.global_msgs_closures.update_value({
                                    move |socket_closures| {
                                        socket_closures.remove(&self.key);
                                    }
                                });
                                SendError::SendError(
                                    e.as_string().unwrap_or(String::from(
                                        "Failed to send web-socket package",
                                    )),
                                )
                            });
                    }
                }

                trace!("ws({}): msg \"{:?}\" pushed to queue", self.ws_url.get_value().unwrap_or("error".to_string()), &package);
                self.global_pending_client_msgs
                    .update_value(|pending| pending.push(bytes));
                Ok(TempSendResult::Queued)
            })
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
pub enum TempSendResult {
    Sent,
    Skipped,
    Queued,
}

#[derive(Debug, Clone)]
pub enum SendResult {
    Sent,
    Queued,
}

#[derive(Error, Debug)]
pub enum SendError {
    #[error("Sending error: {0}.")]
    SendError(String),

    #[error("Failed to serialize client message: {0}.")]
    Serializatoin(String),

    #[error("WebSocket runtime is not initialized.")]
    WsNotInitialized,
}
