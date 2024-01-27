use crate::app::pages::register::GlobalAuthState;
use crate::app::utils::{PageGalleryState, PageProfileState, ScrollSection};
use crate::server::client_msg::ClientMsg;
use crate::server::server_msg::ServerMsg;
use chrono::Utc;
use leptos::{
    create_rw_signal, RwSignal, SignalGetUntracked, SignalUpdateUntracked, SignalWith,
    SignalWithUntracked,
};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub struct GlobalState {
    pub auth: RwSignal<AuthState>,
    pub section: RwSignal<ScrollSection>,
    pub nav_open: RwSignal<bool>,
    pub nav_tran: RwSignal<bool>,
    pub socket_connected: RwSignal<bool>,
    pub socket_send: RwSignal<Rc<dyn Fn(Vec<u8>)>>,
    //pub socket_recv: RwSignal<ServerMsg>,
    pub socket_timestamps: RwSignal<HashMap<&'static str, i64>>,
    pub page_galley: PageGalleryState,
    pub page_profile: PageProfileState,
    pub pages: Pages,
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
}

impl Pages {
    pub fn new() -> Self {
        Self {
            registration: GlobalAuthState::new(),
            login: GlobalAuthState::new(),
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
            socket_send: create_rw_signal(Rc::new(|_| {})),
            socket_connected: create_rw_signal(false),
            //socket_recv: create_rw_signal(ServerMsg::None),
            socket_timestamps: create_rw_signal(HashMap::new()),
            page_galley: PageGalleryState::new(),
            page_profile: PageProfileState::new(),
            pages: Pages::new(),
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

    pub fn socket_send(&self, client_msg: ClientMsg) {
        let name = client_msg.name();
        let bytes = rkyv::to_bytes::<ClientMsg, 256>(&client_msg);
        let Ok(bytes) = bytes else {
            println!(
                "Failed to serialize client msg: {:?}, error: {}",
                &client_msg,
                bytes.err().unwrap()
            );
            return;
        };
        let bytes = bytes.into_vec();
        self.socket_state_used(&name);
        self.socket_send.get_untracked()(bytes);
    }

    pub fn socket_state_is_ready(&self, name: &str) -> bool {
        let socket_state = self
            .socket_timestamps
            .with_untracked(|state| match state.get(name) {
                Some(n) => Some(*n),
                None => None,
            });

        let Some(n) = socket_state else {
            return true;
        };
        let now = Utc::now().timestamp_nanos_opt().unwrap();
        let diff = now - n;
        let is_ready = diff >= 2_000_000_000;
        is_ready
    }

    pub fn socket_state_reset(&self, name: &str) {
        self.socket_timestamps.update_untracked(|state| {
            state.remove(name);
        });
    }

    fn socket_state_used(&self, name: &'static str) {
        self.socket_timestamps.update_untracked(move |state| {
            let Some(socket_state) = state.get_mut(name) else {
                state.insert(name, Utc::now().timestamp_nanos_opt().unwrap());
                return;
            };

            *socket_state = Utc::now().timestamp_nanos_opt().unwrap();
        });
    }
}
