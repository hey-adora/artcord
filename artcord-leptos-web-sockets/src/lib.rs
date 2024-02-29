use std::marker::PhantomData;
use std::{collections::HashMap, fmt::Debug};
use std::rc::Rc;

// use crate::app::global_state::GlobalState;
// use crate::message::server_msg::ServerMsg;
// use crate::server::client_msg::{self, ClientMsg};
use cfg_if::cfg_if;
use chrono::Utc;
use leptos::logging::log;
use leptos::RwSignal;
use leptos::*;
use leptos_use::{
    use_interval_fn, use_websocket_with_options, use_window, UseWebSocketOptions,
    UseWebsocketReturn,
};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use thiserror::Error;

// const WS_TIMEOUT_MS: i64 = 30000;
// const WS_TIMEOUT_MS_MARGIN: i64 = 100;

// This exists just to prevent it sending multiple duplicate requests at once and make ws easier.

#[derive(Clone, Debug)]
pub struct LeptosWebSockets<
    IdType: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + 'static,
    ClientMsg: Clone + Send<IdType> + Debug + 'static,
> {
    pub socket_closures: StoredValue<HashMap<IdType, Rc<dyn Fn(ServerMsg) -> ()>>>,
    pub socket_pending_client_msgs: StoredValue<Vec<u8>>,
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
            socket_closures: StoredValue::new(HashMap::new()),
            socket_pending_client_msgs: StoredValue::new(Vec::new()),
            ws: StoredValue::new(None),
            ws_on_msg: StoredValue::new(None),
            ws_on_err: StoredValue::new(None),
            ws_on_open: StoredValue::new(None),
            ws_on_close: StoredValue::new(None),
            phantom: PhantomData
        }
    }
}

// impl From<TestMsg> for Vec<u8> {
//     fn from(value: TestMsg) -> Self {
//         Vec::new()
//     }
// }

// impl From<Vec<u8>> for TestMsg {
//     fn from(value: Vec<u8>) -> Self {
//         Self{}
//     }
// }

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
                let ws_closures = global_state.socket_closures;
                let ws = global_state.ws;

                ws_on_msg.set_value(Some(Rc::new(Closure::<dyn FnMut(_)>::new(
                    move |e: MessageEvent| Self::on_msg(ws_closures, e)
                ))));

                ws_on_err.set_value(Some(Rc::new(Closure::<dyn FnMut(_)>::new(
                    move |e: ErrorEvent| Self::on_err(e)
                ))));

                ws_on_open.set_value(Some(Rc::new(Closure::<dyn FnMut()>::new(
                    move || Self::on_open()
                ))));

                ws_on_close.set_value(Some(Rc::new(Closure::<dyn FnMut()>::new(
                    move || Self::on_close()
                ))));

                let create_ws = move || -> WebSocket {
                    log!("CONNECTING");
                    let ws = WebSocket::new("ws://localhost:3420").unwrap();
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

                    // ws.set_onmessage(Some((*ws_on_msg.get_untracked()).as_ref().unchecked_ref()));
                    // ws.set_onerror(Some((*ws_on_err.get_untracked()).as_ref().unchecked_ref()));
                    // ws.set_onopen(Some((*ws_on_open.get_untracked()).as_ref().unchecked_ref()));
                    // ws.set_onclose(Some(
                    //     (*ws_on_close.get_untracked()).as_ref().unchecked_ref(),
                    // ));


                    ws
                };

                //log!("AUTH_STATE: {:?}", global_state.auth_is_logged_out());
                // (reconnect_interval.resume)();

                ws.set_value(Some(create_ws()));
                let reconnect_interval = use_interval_fn(
                    move || {
                        let is_closed = ws.with_value(move |ws| {
                            ws.as_ref()
                                .and_then(|ws| Some(ws.ready_state() == WebSocket::CLOSED))
                                .unwrap_or(false)
                        });
                        if is_closed {
                            log!("RECONNECTING");
                            //ws.with_untracked(|ws| {});
                            ws.set_value(Some(create_ws()));
                        }
                    },
                    1000,
                );
            }
        }
    }

    fn create_group() -> WsGroup<IdType, ServerMsg, ClientMsg> {
        let global_state = use_context::<LeptosWebSockets<IdType, ServerMsg, ClientMsg>>().expect("Failed to provide global state");
        let ws_closures = global_state.socket_closures;
        let ws = global_state.ws;
        WsGroup::<IdType, ServerMsg, ClientMsg>::new(Self::generate_key(), ws_closures, ws)
    }

    fn generate_key() -> IdType;


    fn on_open() {
        log!("CONNECTED");
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
        //log!("ONG MSG {:?}", vec);
        if bytes.is_empty() {
            log!("Empty byte msg received.");
            return;
        };

        let server_msg = ServerMsg::recv_from_vec(&bytes);
        let Ok((id, server_msg)) = server_msg else {
            log!("Error decoding msg: {}", server_msg.err().unwrap());
            return;
        };

        //log!("{:#?}", &server_msg);

        Self::execute(closures, &id, server_msg);
        // if id != 0 {
        //     global_state.execute(id, server_msg);
        // } else {
        //     log!("IDDDDDDDD 0");
        // }
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

// enum TestMsg {

// }

// pub trait LWSSend {
//     fn to_vec(&self, id: &str) -> Result<Vec<u8>, String>;
// }

// pub trait LeptosWebSocketsReceive {
//     fn from_vec(bytes: &[u8]) -> Result<Self, String> where Self: std::marker::Sized;
// }

// pub trait LWSExecute {
//     fn from_vec(bytes: &[u8]) -> Result<Self, String> where Self: std::marker::Sized;
// }

// impl LeptosWebSockets {
//     pub fn send<ClientMsg: LWSSend, ServerMsg: TryFrom<Vec<u8>>>(client_msg: &ClientMsg, on_receive: impl Fn(ServerMsg) -> () + 'static) {
//         let leptos_web_sockets = use_context::<LeptosWebSockets>();
//         let Ok(leptos_web_sockets) = leptos_web_sockets else {
//             log!("LeptosWebSockets are not initialized.");
//             return;
//         };
//         let uuid: u128 = uuid::Uuid::new_v4().to_u128_le();

//         leptos_web_sockets.ws.with_value(|ws| {
//             let Some(ws) = ws else {
//                 log!("WS is not initialized");
//                 return;
//             };

//             let bytes = client_msg.to_vec(uuid);
//             let Ok(bytes) = bytes else {
//                 println!(
//                     "Failed to serialize client msg: {:?}, error: {}",
//                     &client_msg,
//                     bytes.err().unwrap()
//                 );
//                 return;
//             };

//             leptos_web_sockets.socket_closures.update_value({
//                 let socket_closures = leptos_web_sockets.socket_closures.clone();
//                 move |socket_closures| {
//                     let f = Rc::new(move |server_msg| {
//                         on_receive(server_msg);
//                     });
//                     socket_closures.insert(leptos_web_sockets.uuid, f);
//                 }
//             });

//             log!("SENT SOME MSG");
//             ws.send_with_u8_array(&bytes);
//         });
//     }
// }

#[derive(Clone, Debug)]
pub struct WsGroup<
    IdType: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive<IdType> + 'static,
    ClientMsg: Clone + Send<IdType> + Debug + 'static,
> {
    socket_closures: StoredValue<HashMap<IdType, Rc<dyn Fn(ServerMsg) -> ()>>>,
    //socket_send_fn: StoredValue<Rc<dyn Fn(Vec<u8>)>>,
    ws: StoredValue<Option<WebSocket>>,
    key: IdType,
    phantom: PhantomData<ClientMsg>
}

impl <
    IdType: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug + 'static,
    ServerMsg: Clone + Receive<IdType> + 'static,
    ClientMsg: Clone + Send<IdType> + Debug + 'static,
> WsGroup<IdType, ServerMsg, ClientMsg> {
    pub fn new(
        key: IdType,
        socket_closures: StoredValue<HashMap<IdType, Rc<dyn Fn(ServerMsg) -> ()>>>,
        ws: StoredValue<Option<WebSocket>>,
    ) -> Self {
        //let uuid: u128 = uuid::Uuid::new_v4().to_u128_le();

        Self {
            socket_closures,
            ws,
            key,
            phantom: PhantomData
        }
    }

    pub fn send(&self, client_msg: &ClientMsg, on_receive: impl Fn(ServerMsg) -> () + 'static) -> Result<(), SendError> {
        self.ws.with_value(|ws| -> Result<(), SendError> {
            let ws = ws.as_ref().ok_or_else(|| SendError::WsNotInitialized)?;
            let bytes = client_msg.send_as_vec(&self.key).or_else(|e| Err(SendError::SendError(e)))?;
            self.socket_closures.update_value({
                move |socket_closures| {
                    let a = Rc::new(move |server_msg| {
                        on_receive(server_msg);
                    });
                    socket_closures.insert(self.key.clone(), a);
                }
            });
            ws.send_with_u8_array(&bytes).or_else(|e| {
                self.socket_closures.update_value({
                    move |socket_closures| {
                        socket_closures.remove(&self.key);
                    }
                });
                Err(SendError::SendError(e.as_string().unwrap_or(String::from("Failed to send web-socket package"))))
            })
        })
    }
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