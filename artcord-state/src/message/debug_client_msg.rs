use tracing::{error, info, trace, warn};
use serde::{Deserialize, Serialize};

use super::debug_msg_key::DebugMsgKey;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub enum DebugClientMsg {
    Ready,
}

impl artcord_leptos_web_sockets::Send<DebugMsgKey> for DebugClientMsg {
    fn send_as_vec(&self, id: &DebugMsgKey) -> Result<Vec<u8>, String> {
        self.as_vec(id).map_err(|e| e.to_string())
    }
}

impl DebugClientMsg {
    pub fn as_vec(&self, id: &DebugMsgKey) -> Result<Vec<u8>, bincode::Error> {
        let object = (*id, *self);
        let result: Result<Vec<u8>, Box<bincode::ErrorKind>> = bincode::serialize::<(DebugMsgKey, DebugClientMsg)>(&object);
        trace!("debug client msg serialized from {:?} {:?}", &object, &result);
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(DebugMsgKey, DebugClientMsg), bincode::Error> {
        let result: Result<(DebugMsgKey, DebugClientMsg), Box<bincode::ErrorKind>> = bincode::deserialize::<(DebugMsgKey, DebugClientMsg)>(bytes);
        trace!("debug client msg deserialized from {:?} to {:?}", bytes, &result);
        result
    }
}