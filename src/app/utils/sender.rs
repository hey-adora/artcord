use std::collections::HashMap;
use std::rc::Rc;

use chrono::Utc;
use leptos::RwSignal;
use leptos::logging::log;
use leptos::*;
use crate::app::global_state::GlobalState;
use crate::message::server_msg::ServerMsg;
use crate::server::client_msg::{self, ClientMsg};
use web_sys::WebSocket;

// const WS_TIMEOUT_MS: i64 = 30000;
// const WS_TIMEOUT_MS_MARGIN: i64 = 100;

// This exists just to prevent it sending multiple duplicate requests at once and make ws easier.
#[derive(Copy, Clone, Debug)]
pub struct WsSender {
    socket_closures: StoredValue<HashMap<u128, Rc<dyn Fn(ServerMsg)->() >>>,
    //socket_send_fn: StoredValue<Rc<dyn Fn(Vec<u8>)>>,
    ws: StoredValue<Option<WebSocket>>,
    uuid: u128
}

impl WsSender {
    pub fn new(socket_closures: StoredValue<HashMap<u128, Rc<dyn Fn(ServerMsg)->() >>>, ws: StoredValue<Option<WebSocket>>) -> Self {
        let uuid: u128 = uuid::Uuid::new_v4().to_u128_le();

        Self {
            socket_closures,
            ws,
            uuid
        }
    }

    pub fn send(&self, client_msg: &ClientMsg, on_receive: impl Fn(ServerMsg) -> () + 'static) {
        self.socket_closures.update_value({
            let socket_closures = self.socket_closures.clone();
            move |socket_closures| {
                let a = Rc::new(move |server_msg| {
                    on_receive(server_msg);
                });
                //log!("ATTACHED: {:?}", socket_closures.len());
                socket_closures.insert(self.uuid, a);
            }
        });
        
        self.send_with(self.uuid, client_msg);
    }

    pub fn is_loading(&self) -> bool {
        self.socket_closures.with_value(|closures|{
            closures.contains_key(&self.uuid)
        })
    }

    // pub fn create_sender(&self) -> (Rc<dyn Fn() -> bool + 'static>, Rc<dyn Fn(&ClientMsg, fn(ServerMsg) -> ()) -> () + 'static>) {
        

    //     let send_msg = move |client_msg: &ClientMsg, on_receive: fn(ServerMsg) -> ()| {
            
    //     };

    //     let is_loading = move || -> bool {
            
    //     };

    //     (Rc::new(is_loading), Rc::new(send_msg))
    // }

    fn send_with(&self, id: u128, client_msg: &ClientMsg) {
        self.ws.with_value(|ws| {
            let Some(ws) = ws else {
                log!("WS is not initialized");
                return;
             };
    
             let bytes = client_msg.as_vec(id);
             let Ok(bytes) = bytes else {
                 println!(
                     "Failed to serialize client msg: {:?}, error: {}",
                     &client_msg,
                     bytes.err().unwrap()
                 );
                 return;
             };
             log!("SENT SOME MSG");
             ws.send_with_u8_array(&bytes);
        });
     }
}


// #[derive(Debug, Clone)]
// pub struct Fender<S: Clone + 'static, E: Clone + 'static> {
//     pub value: RwSignal<S>,
//     pub error: RwSignal<E>,
//     pub was_not_sent_yet: RwSignal<bool>,
//     pub state: RwSignal<SenderState>,
//     old_state: RwSignal<SenderState>,
//    // uuid: u128,
//     socket_send_fn: RwSignal<Rc<dyn Fn(Vec<u8>)>>,
//     socket_closures: RwSignal<HashMap<u128, Rc<dyn Fn(ServerMsg)->() >>>,
//     last_used: RwSignal<i64>,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub enum SenderState {
//     Connecting,
//     Ready,
//     Loading,
//     Succesfull,
//     Error,
// }

// impl<S: Clone + 'static, E: Clone + 'static> Copy for Fender<S, E> {}

// impl <S: Default + Clone + 'static, E: Default + Clone + 'static> Fender<S, E> {

//     pub fn new(is_connected: bool, socket_closures: RwSignal<HashMap<u128, Rc<dyn Fn(ServerMsg)>>>,  ) -> Self {
//         let global_state = use_context::<GlobalState>().expect("Failed to provide socket_bs state");
//         let is_connected = global_state.socket_connected;
//         let socket_closures: RwSignal<HashMap<u128, Rc<dyn Fn(ServerMsg)>>> = global_state.socket_closures;
        
//         let default_state = if is_connected.get_untracked() { SenderState::Ready } else { SenderState::Connecting };
//         let old_state = RwSignal::new(default_state.clone());
//         let state = RwSignal::new(default_state);
//         let value = RwSignal::new(S::default());
//         let error = RwSignal::new(E::default());
//         let was_not_sent_yet = RwSignal::new(true);
//         let socket_send_fn = global_state.socket_send_fn;
//         let last_used = RwSignal::new(0i64);

//         create_effect(move |_| {
//             let is_connected = is_connected.get();
//             if is_connected {
//                 if old_state.with_untracked(move |old_state| *old_state != SenderState::Connecting) {
//                     state.set(old_state.get_untracked());
//                 } else {
//                     state.set(SenderState::Ready);
//                 }
//             } else {
//                 old_state.set(state.get_untracked());
//                 state.set(SenderState::Connecting);
//             }
//         });

//         Self {
//             //uuid,
//             old_state,
//             state,
//             value,
//             error,
//             was_not_sent_yet,
//             socket_send_fn,
//             last_used,
//             socket_closures
//         }
//     }

//     pub fn send<F: Fn(ServerMsg) -> Option<SenderState> + 'static>(&self, client_msg: &ClientMsg, on_receive: F) {
//         if self.state.get() == SenderState::Connecting {
//             return;
//         }

//         self.last_used.set(Utc::now().timestamp_millis());
//         let uuid = uuid::Uuid::new_v4().to_u128_le();
//         self.socket_closures.update_untracked({
//             let socket_closures = self.socket_closures.clone();
//             let state = self.state.clone();
//             move |socket_closures| {
//                 let a = Rc::new(move |server_msg| {
//                     let result = on_receive(server_msg);
//                     if let Some(result) = result {
//                         state.set(result);
//                     }
         
//                 });
//                 log!("ATTACHED: {:?}", socket_closures.len());
//                 socket_closures.insert(uuid, a);
//             }
//         });

//         // on_cleanup({
//         //     let socket_closures = self.socket_closures.clone();
//         //     move || {
//         //         socket_closures.update_untracked(move |socket_closures| {
//         //             socket_closures.remove(&uuid);
//         //             log!("DETACHING: {:?}", socket_closures.len());
//         //         });
//         //     }
//         // });

//         let bytes = client_msg.as_vec(uuid);
//         let Ok(bytes) = bytes else {
//             println!(
//                 "Failed to serialize client msg: {:?}, error: {}",
//                 &client_msg,
//                 bytes.err().unwrap()
//             );
//             return;
//         };
//         self.socket_send_fn.get_untracked()(bytes);
//     }

    


//     // pub fn create_sender<F: Fn(ServerMsg) -> Option<SenderState> + Copy + 'static>(on_receive: F) -> Rc<dyn Fn(&ClientMsg)> {
//     //     let uuid = uuid::Uuid::new_v4().to_u128_le();
//     //     let global_state = use_context::<GlobalState>().expect("Failed to provide socket_bs state");
//     //     let msgs = global_state.socket_closures;
//     //     let is_connected = global_state.socket_connected;
//     //     let default_state = if is_connected.get_untracked() { SenderState::Ready } else { SenderState::Connecting };
//     //     let old_state = RwSignal::new(default_state.clone());
//     //     let state = RwSignal::new(default_state);

//     //     let send_msg = move |client_msg: &ClientMsg| {
//     //         global_state.socket_send(uuid, client_msg);
//     //     };

//     //     create_effect(move |_| {
//     //         log!("ATTACHING: {:?}", msgs.with_untracked(|msgs|msgs.len()));
//     //         msgs.update_untracked(move |msgs| {
//     //             let a = Rc::new(move |server_msg| {
//     //                 let result = on_receive(server_msg);
//     //                 if let Some(result) = result {
//     //                     state.set(result);
//     //                 }
//     //             });
//     //             msgs.insert(uuid, a);
//     //         });
//     //         log!("ATTACHED: {:?}", msgs.with_untracked(|msgs|msgs.len()));
//     //     });

//     //     create_effect(move |_| {
//     //         let is_connected = is_connected.get();
//     //         if is_connected {
//     //             if old_state.with_untracked(move |old_state| *old_state != SenderState::Connecting) {
//     //                 state.set(old_state.get_untracked());
//     //             } else {
//     //                 state.set(SenderState::Ready);
//     //             }
//     //         } else {
//     //             old_state.set(state.get_untracked());
//     //             state.set(SenderState::Connecting);
//     //         }
//     //     });
        
        
//     //     on_cleanup(move || {
//     //         msgs.update(move |msgs| {
//     //             msgs.remove(&uuid);
//     //         });
//     //         log!("DETACHING: {:?}", msgs.with_untracked(|msgs|msgs.len()));
//     //     });



//     //     Rc::new(send_msg)
//     // }
// }

// // pub fn send(client_msg: &ClientMsg) {
// //     let uuid = uuid::Uuid::new_v4().to_u128_le();
// //     self.socket_closures.update_untracked({
// //         let socket_closures = self.socket_closures.clone();
// //         let state = self.state.clone();
// //         move |socket_closures| {
// //             let a = Rc::new(move |server_msg| {
// //                 let result = on_receive(server_msg);
// //                 if let Some(result) = result {
// //                     state.set(result);
// //                 }
     
// //             });
// //             log!("ATTACHED: {:?}", socket_closures.len());
// //             socket_closures.insert(uuid, a);
// //         }
// //     });
// // }

// // pub fn execute(id: u128, server_msg: ServerMsg) {
// //     let global_state = use_context::<GlobalState>().expect("Failed to provide socket_bs state");
// //     let msgs: RwSignal<HashMap<u128, Rc<dyn Fn(ServerMsg)>>> = global_state.socket_closures;

// //     msgs.update_untracked(move |msgs| {
// //         let Some(f) = msgs.get(&id) else {
// //             log!("Fn not found for {}", id);
// //             return;
// //         };
        
// //         f(server_msg);

// //         msgs.remove(&id);
// //     });
// // }