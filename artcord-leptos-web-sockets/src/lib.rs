use std::marker::PhantomData;
use std::{collections::HashMap, fmt::Debug};
use std::rc::Rc;

use cfg_if::cfg_if;
use leptos::logging::log;
use leptos::*;
use leptos_use::use_window;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use thiserror::Error;

// const WS_TIMEOUT_MS: i64 = 30000;
// const WS_TIMEOUT_MS_MARGIN: i64 = 100;

#[derive(Clone, Debug)]
pub struct LeptosWebSockets<
    IdType: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + 'static,
    ClientMsg: Clone + Send<IdType> + Debug + 'static,
> {
    pub global_msgs_closures: StoredValue<HashMap<IdType, Rc<dyn Fn(ServerMsg) -> ()>>>,
    pub global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    pub ws: StoredValue<Option<WebSocket>>,
    pub ws_on_msg: StoredValue<Option<Rc<Closure<dyn FnMut(MessageEvent)>>>>,
    pub ws_on_err: StoredValue<Option<Rc<Closure<dyn FnMut(ErrorEvent)>>>>,
    pub ws_on_open: StoredValue<Option<Rc<Closure<dyn FnMut()>>>>,
    pub ws_on_close: StoredValue<Option<Rc<Closure<dyn FnMut()>>>>,
    phantom: PhantomData<ClientMsg>
}

impl<
        IdType: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + 'static,
        ServerMsg: Clone + 'static,
        ClientMsg: Clone + Send<IdType> + Debug + 'static,
    > LeptosWebSockets<IdType, ServerMsg, ClientMsg>
{
    pub fn new() -> Self {
        Self {
            global_msgs_closures: StoredValue::new(HashMap::new()),
            global_pending_client_msgs: StoredValue::new(Vec::new()),
            ws: StoredValue::new(None),
            ws_on_msg: StoredValue::new(None),
            ws_on_err: StoredValue::new(None),
            ws_on_open: StoredValue::new(None),
            ws_on_close: StoredValue::new(None),
            phantom: PhantomData
        }
    }
}

pub trait Send<IdType: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + 'static> {
    fn send_as_vec(&self, id: &IdType) -> Result<Vec<u8>, String>;
}

pub trait Receive<IdType: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + 'static> {
    fn recv_from_vec(bytes: &[u8]) -> Result<(IdType, Self), String>
    where
        Self: std::marker::Sized;
}

pub trait Runtime<
    IdType: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive<IdType> + 'static,
    ClientMsg: Clone + Send<IdType> + Debug + 'static,
>
{
    fn new() {
        provide_context(LeptosWebSockets::<IdType, ServerMsg, ClientMsg>::new());

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let global_state = use_context::<LeptosWebSockets<IdType, ServerMsg, ClientMsg>>().expect("Failed to provide global state");

                let ws_on_msg = global_state.ws_on_msg;
                let ws_on_err = global_state.ws_on_err;
                let ws_on_open = global_state.ws_on_open;
                let ws_on_close = global_state.ws_on_close;
                let ws_closures = global_state.global_msgs_closures;
                let ws_pending = global_state.global_pending_client_msgs;
                let ws = global_state.ws;

                ws_on_msg.set_value(Some(Rc::new(Closure::<dyn FnMut(_)>::new(
                    move |e: MessageEvent| Self::on_msg(ws_closures, e)
                ))));

                ws_on_err.set_value(Some(Rc::new(Closure::<dyn FnMut(_)>::new(
                    move |e: ErrorEvent| Self::on_err(e)
                ))));

                ws_on_open.set_value(Some(Rc::new(Closure::<dyn FnMut()>::new(
                    move || Self::on_open(ws_pending, ws)
                ))));

                ws_on_close.set_value(Some(Rc::new(Closure::<dyn FnMut()>::new(
                    move || Self::on_close()
                ))));

                let create_ws = move || -> WebSocket {
                    log!("CONNECTING");
                    let ws = WebSocket::new(&get_ws_path()).unwrap();
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
                };

                ws.set_value(Some(create_ws()));
                let reconnect_interval = leptos_use::use_interval_fn(
                    move || {
                        let is_closed = ws.with_value(move |ws| {
                            ws.as_ref()
                                .and_then(|ws| Some(ws.ready_state() == WebSocket::CLOSED))
                                .unwrap_or(false)
                        });
                        if is_closed {
                            log!("RECONNECTING");
                            ws.set_value(Some(create_ws()));
                        }
                    },
                    1000,
                );
            }
        }
    }

    fn new_singleton() -> WsSingleton<IdType, ServerMsg, ClientMsg> {
        let global_state = use_context::<LeptosWebSockets<IdType, ServerMsg, ClientMsg>>().expect("Failed to provide global state");
        let ws_closures = global_state.global_msgs_closures;
        let ws = global_state.ws;
        let socket_pending_client_msgs = global_state.global_pending_client_msgs;
        WsSingleton::<IdType, ServerMsg, ClientMsg>::new(Self::generate_key(), ws_closures, ws, socket_pending_client_msgs)
    }

    fn generate_key() -> IdType;


    fn on_open(socket_pending_client_msgs: StoredValue<Vec<Vec<u8>>>, ws: StoredValue<Option<WebSocket>>) {
        log!("CONNECTED");
        Self::flush_pending_client_msgs(socket_pending_client_msgs, ws);
    }

    fn on_close() {
        log!("DISCONNECTED");
    }

    fn on_err(e: ErrorEvent) {
        log!("WS ERROR: {:?}", e);
    }

    fn on_msg(
        closures: StoredValue<HashMap<IdType, Rc<dyn Fn(ServerMsg) -> ()>>>,
        e: MessageEvent,
    ) {
        let data = e.data().dyn_into::<js_sys::ArrayBuffer>();
        let Ok(data) = data else {
            return;
        };
        let array = js_sys::Uint8Array::new(&data);
        let bytes: Vec<u8> = array.to_vec();

        if bytes.is_empty() {
            log!("Empty byte msg received.");
            return;
        };

        let server_msg = ServerMsg::recv_from_vec(&bytes);
        let Ok((id, server_msg)) = server_msg else {
            log!("Error decoding msg: {}", server_msg.err().unwrap());
            return;
        };

        Self::execute(closures, &id, server_msg);
    }

    
    fn flush_pending_client_msgs(socket_pending_client_msgs: StoredValue<Vec<Vec<u8>>>, ws: StoredValue<Option<WebSocket>>) {
        ws.with_value(|ws| {
            if let Some(ws) = ws {
                socket_pending_client_msgs.update_value(|msgs| {
                    let mut index: usize = 0;
                    for msg in msgs.iter() {
                        let result = ws.send_with_u8_array(msg);
                        if result.is_err()  {
                            break;
                        }  
                        index += 1;
                    }
                    if index < msgs.len() && index > 0 {
                        *msgs = (&msgs[index..]).to_vec();
                    }
                });
            } else {
                log!("WebSockets are not initialized.");
            }
       
        });
    }

    fn execute(
        closures: StoredValue<HashMap<IdType, Rc<dyn Fn(ServerMsg) -> ()>>>,
        id: &IdType,
        server_msg: ServerMsg,
    ) {
        closures.update_value(move |socket_closures| {
            let Some(f) = socket_closures.get(id) else {
                log!("Fn not found for {:?}", id);
                return;
            };

            f(server_msg);

            socket_closures.remove(id);
        });
    }


}

pub fn get_ws_path() -> String {
    let default = String::from("wss://artcord.uk.to:3420");
    let mut output = String::new();
    let window = &*use_window();
    let Some(window) = window else {
        log!("Failed to get window for get_ws_path, using default ws path: {}", default);
        return default;
    };
    //let location = use_location();
    let protocol = window.location().protocol();
    let Ok(protocol) = protocol else {
        log!("Failed to get window for protocol, using default ws path: {}", default);
        return default;
    };
    if protocol == "http:" {
        output.push_str("ws://");
    } else {
        output.push_str("wss://");
    }
    let hostname = window.location().hostname();
    let Ok(hostname) = hostname else {
        log!("Failed to get window for hostname, using default ws path: {}", default);
        return default;
    };
    output.push_str(&format!("{}:3420", hostname));

    output
}



#[derive(Clone, Debug)]
pub struct WsSingleton<
    IdType: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive<IdType> + 'static,
    ClientMsg: Clone + Send<IdType> + Debug + 'static,
> {
    global_msgs_closures: StoredValue<HashMap<IdType, Rc<dyn Fn(ServerMsg) -> ()>>>,
    global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    //socket_send_fn: StoredValue<Rc<dyn Fn(Vec<u8>)>>,
    ws: StoredValue<Option<WebSocket>>,
    key: IdType,
    phantom: PhantomData<ClientMsg>,
}

impl <
    IdType: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive<IdType> + 'static,
    ClientMsg: Clone + Send<IdType> + Debug + 'static,
> WsSingleton<IdType, ServerMsg, ClientMsg> {
    pub fn new(
        key: IdType,
        global_msgs_closures: StoredValue<HashMap<IdType, Rc<dyn Fn(ServerMsg) -> ()>>>,
        ws: StoredValue<Option<WebSocket>>,
        global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    ) -> Self {
        on_cleanup({
            let key = key.clone();
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
            key,
            phantom: PhantomData
        }
    }

    pub fn send_once(&self, client_msg: &ClientMsg, on_receive: impl Fn(ServerMsg) -> () + 'static) -> Result<SendResult, SendError> {
        self.ws.with_value(|ws| -> Result<SendResult, SendError> {
            let ws = ws.as_ref().ok_or_else(|| SendError::WsNotInitialized)?;
            let bytes = client_msg.send_as_vec(&self.key).or_else(|e| Err(SendError::SendError(e)))?;
            
            let exists = self.global_msgs_closures.with_value(|socket_closures| {
                socket_closures.contains_key(&self.key)
            });
            if exists {
                return Ok(SendResult::Skipped);
            }

            self.global_msgs_closures.update_value({
                move |global_msgs_closures| {
                    let new_msg_closure = Rc::new(move |server_msg| {
                        on_receive(server_msg);
                    });
                    global_msgs_closures.insert(self.key.clone(), new_msg_closure);
                }
            });

            let is_open = self.ws.with_value(move |ws| {
                ws.as_ref()
                    .and_then(|ws| Some(ws.ready_state() == WebSocket::OPEN))
                    .unwrap_or(false)
            });

            if is_open {
                ws.send_with_u8_array(&bytes).and_then(|_|Ok(SendResult::Sent)).or_else(|e| {
                    self.global_msgs_closures.update_value({
                        move |socket_closures| {
                            socket_closures.remove(&self.key);
                        }
                    });
                    Err(SendError::SendError(e.as_string().unwrap_or(String::from("Failed to send web-socket package"))))
                })
            } else {
                self.global_pending_client_msgs.update_value(|pending| {
                    pending.push(bytes)
                });
                Ok(SendResult::Queued)
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum SendResult {
    Sent,
    Skipped,
    Queued
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