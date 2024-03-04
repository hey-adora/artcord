use artcord_leptos_web_sockets::Runtime;

use super::{client_msg_wrap::ClientMsgWrap, server_msg_wrap::ServerMsgWrap};

pub struct WsRuntime;

impl Runtime<u128, ServerMsgWrap, ClientMsgWrap> for WsRuntime {
    fn generate_key() -> u128 {
        uuid::Uuid::new_v4().to_u128_le()
    }
}