use tracing::{error, info, trace, warn};
use serde::{Deserialize, Serialize};

use super::debug_msg_key::DebugMsgKey;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub enum DebugServerMsg {
    Restart,
}

impl artcord_leptos_web_sockets::Receive<DebugMsgKey> for DebugServerMsg {
    fn recv_from_vec(bytes: &[u8]) -> Result<(DebugMsgKey, Self), String> where Self: std::marker::Sized {
        DebugServerMsg::from_bytes(bytes).map_err(|e| e.to_string())
    }
}

impl DebugServerMsg {
    pub fn from_bytes(bytes: &[u8]) -> Result<(DebugMsgKey, Self), bincode::Error> {
        let result = bincode::deserialize::<(DebugMsgKey, DebugServerMsg)>(bytes);
        trace!("debug server msg deserialized from {:?} to {:?}", bytes, &result);
        result
    }

    pub fn as_bytes(&self, id: &DebugMsgKey) -> Result<Vec<u8>, bincode::Error> {
        let object = (*id, *self);
        let result = bincode::serialize::<(DebugMsgKey, DebugServerMsg)>(&object);
        trace!("debug server msg serialized from {:?} {:?}", &object, &result);
        result
    }
}