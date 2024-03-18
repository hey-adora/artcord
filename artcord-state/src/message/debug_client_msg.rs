use tracing::{error, info, trace, warn};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum DebugClientMsg {
    Ready,
}

impl artcord_leptos_web_sockets::Send<u128> for DebugClientMsg {
    fn send_as_vec(&self, id: &u128) -> Result<Vec<u8>, String> {
        self.as_vec(*id).map_err(|e| e.to_string())
    }
}

impl DebugClientMsg {
    pub fn as_vec(&self, id: u128) -> Result<Vec<u8>, bincode::Error> {
        let object = (id, self.clone());
        let result: Result<Vec<u8>, Box<bincode::ErrorKind>> = bincode::serialize::<(u128, DebugClientMsg)>(&object);
        trace!("debug client msg serialized from {:?} {:?}", &object, &result);
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(u128, DebugClientMsg), bincode::Error> {
        let result: Result<(u128, DebugClientMsg), Box<bincode::ErrorKind>> = bincode::deserialize::<(u128, DebugClientMsg)>(bytes);
        trace!("debug client msg deserialized from {:?} to {:?}", bytes, &result);
        result
    }
}