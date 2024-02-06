use crate::app::pages::register::GlobalAuthState;
use crate::app::utils::{PageProfileState, ScrollSection};
use crate::message::server_msg::ServerMsg;
use crate::server::client_msg::ClientMsg;
use chrono::Utc;
use leptos::{
    create_rw_signal, RwSignal, SignalGetUntracked, SignalUpdateUntracked, SignalWith,
    SignalWithUntracked,
};
use std::collections::HashMap;
use std::rc::Rc;
use leptos::logging::log;

use super::pages::gallery::GalleryPageState;
use super::utils::sender::WsSender;

#[derive(Copy, Clone, Debug)]
pub struct GlobalState {
    pub auth: RwSignal<AuthState>,
    pub section: RwSignal<ScrollSection>,
    pub nav_open: RwSignal<bool>,
    pub nav_tran: RwSignal<bool>,
    pub socket_connected: RwSignal<bool>,
    pub socket_send_fn: RwSignal<Rc<dyn Fn(Vec<u8>)>>,
    //pub socket_recv: RwSignal<ServerMsg>,
    pub socket_timestamps: RwSignal<HashMap<&'static str, i64>>,
  //  pub page_galley: PageGalleryState,
    pub page_profile: PageProfileState,
    pub pages: Pages,
    pub socket_closures: RwSignal<HashMap<u128, Rc<dyn Fn(ServerMsg)->() >>>,
   // pub socket_timeouts: RwSignal<HashMap<u128, (RwSignal<SenderState>, RwSignal<String>)>>
}

// #[derive(Clone, Debug)]
// pub struct Auth {
//     user_id: String,
// }

#[derive(Clone, Debug)]
pub enum AuthState {
    Processing,
    LoggedIn { user_id: String },
    LoggedOut,
}

#[derive(Copy, Clone, Debug)]
pub struct Pages {
    pub registration: GlobalAuthState,
    pub login: GlobalAuthState,
    pub gallery: GalleryPageState,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            registration: GlobalAuthState::new(),
            login: GlobalAuthState::new(),
            gallery: GalleryPageState::new()
        }
    }
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            auth: create_rw_signal(AuthState::Processing),
            section: create_rw_signal(ScrollSection::Home),
            nav_open: create_rw_signal(false),
            nav_tran: create_rw_signal(true),
            socket_send_fn: create_rw_signal(Rc::new(|_| {})),
            socket_connected: create_rw_signal(false),
            //socket_recv: create_rw_signal(ServerMsg::None),
            socket_timestamps: create_rw_signal(HashMap::new()),
            //page_galley: PageGalleryState::new(),
            page_profile: PageProfileState::new(),
            pages: Pages::new(),
            socket_closures: RwSignal::new(HashMap::new()),
            //socket_timeouts: RwSignal::new(HashMap::new())
        }
    }

    pub fn auth_is_processing(&self) -> bool {
        self.auth.with(|a| match a {
            AuthState::Processing => true,
            _ => false,
        })
    }

    pub fn auth_is_logged_in(&self) -> bool {
        self.auth.with(|a| match a {
            AuthState::LoggedIn { user_id } => true,
            _ => false,
        })
    }

    pub fn auth_is_logged_out(&self) -> bool {
        self.auth.with(|a| match a {
            AuthState::LoggedOut => true,
            _ => false,
        })
    }

    pub fn send_with(&self, id: u128, client_msg: &ClientMsg) {
       // let name = client_msg.name();
        let bytes = client_msg.as_vec(id);
        //let bytes = rkyv::to_bytes::<ClientMsg, 256>(&client_msg);
        let Ok(bytes) = bytes else {
            println!(
                "Failed to serialize client msg: {:?}, error: {}",
                &client_msg,
                bytes.err().unwrap()
            );
            return;
        };
        //let bytes = bytes.into_vec();
       // self.socket_state_used(&name);
        self.socket_send_fn.get_untracked()(bytes);
    }

    pub fn create_sender(&self) -> WsSender {
        WsSender::new(self.socket_closures, self.socket_send_fn)
    }

    // pub fn create_sender(&self) -> (Rc<dyn Fn() -> bool + 'static>, Rc<dyn Fn(&ClientMsg, fn(ServerMsg) -> ()) -> () + 'static>) {
    //     let uuid = uuid::Uuid::new_v4().to_u128_le();

    //     let send_msg = move |client_msg: &ClientMsg, on_receive: impl Fn(ServerMsg) -> ()| {
    //         self.socket_closures.update_untracked({
    //             let socket_closures = self.socket_closures.clone();
    //             move |socket_closures| {
    //                 let a = Rc::new(move |server_msg| {
    //                     on_receive(server_msg);
             
    //                 });
    //                 log!("ATTACHED: {:?}", socket_closures.len());
    //                 socket_closures.insert(uuid, a);
    //             }
    //         });
            
    //         self.send_with(uuid, client_msg);
    //     };

    //     let is_loading = move || -> bool {
    //         self.socket_closures.with(|closures|{
    //             closures.contains_key(&uuid)
    //         })
    //     };

    //     (Rc::new(is_loading), Rc::new(send_msg))
    // }
    
    pub fn execute(&self, id: u128, server_msg: ServerMsg) {
        //let global_state = use_context::<GlobalState>().expect("Failed to provide socket_bs state");
        //let msgs: RwSignal<HashMap<u128, Rc<dyn Fn(ServerMsg)>>> = global_state.socket_closures;
    
        self.socket_closures.update_untracked(move |socket_closures| {
            let Some(f) = socket_closures.get(&id) else {
                log!("Fn not found for {}", id);
                return;
            };
            
            f(server_msg);
    
            socket_closures.remove(&id);
        });
    }

    // pub fn socket_state_is_ready(&self, name: &str) -> bool {
    //     let socket_state = self
    //         .socket_timestamps
    //         .with_untracked(|state| match state.get(name) {
    //             Some(n) => Some(*n),
    //             None => None,
    //         });

    //     let Some(n) = socket_state else {
    //         return true;
    //     };
    //     let now = Utc::now().timestamp_nanos_opt().unwrap();
    //     let diff = now - n;
    //     let is_ready = diff >= 2_000_000_000;
    //     is_ready
    // }

    // pub fn socket_state_reset(&self, name: &str) {
    //     self.socket_timestamps.update_untracked(|state| {
    //         state.remove(name);
    //     });
    // }

    // fn socket_state_used(&self, name: &'static str) {
    //     self.socket_timestamps.update_untracked(move |state| {
    //         let Some(socket_state) = state.get_mut(name) else {
    //             state.insert(name, Utc::now().timestamp_nanos_opt().unwrap());
    //             return;
    //         };

    //         *socket_state = Utc::now().timestamp_nanos_opt().unwrap();
    //     });
    // }
}
