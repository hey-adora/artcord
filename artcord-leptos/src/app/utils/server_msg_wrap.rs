use artcord_state::message::server_msg::ServerMsg;

#[derive(Debug, PartialEq, Clone)]
pub struct ServerMsgWrap(pub ServerMsg);

impl artcord_leptos_web_sockets::Receive<u128> for ServerMsgWrap {
    fn recv_from_vec(bytes: &[u8]) -> Result<(u128, Self), String> where Self: std::marker::Sized {
        ServerMsg::from_bytes(&bytes).and_then(|msg| Ok((msg.0, ServerMsgWrap(msg.1)))).or_else(|e| Err(e.to_string()))
    }
}
