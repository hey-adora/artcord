use artcord_state::message::client_msg::ClientMsg;

#[derive(Debug, PartialEq, Clone)]
pub struct ClientMsgWrap(pub ClientMsg);

impl artcord_leptos_web_sockets::Send<u128> for ClientMsgWrap {
    fn send_as_vec(&self, id: &u128) -> Result<Vec<u8>, String> {
        self.0.as_vec(*id).or_else(|e| Err(e.to_string()))
    }
}