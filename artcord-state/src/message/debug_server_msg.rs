use tracing::{error, info, trace, warn};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum DebugServerMsg {
    Restart,
}

impl artcord_leptos_web_sockets::Receive<u128> for DebugServerMsg {
    fn recv_from_vec(bytes: &[u8]) -> Result<(u128, Self), String> where Self: std::marker::Sized {
        DebugServerMsg::from_bytes(bytes).map_err(|e| e.to_string())
    }
}

impl DebugServerMsg {
    pub fn from_bytes(bytes: &[u8]) -> Result<(u128, Self), bincode::Error> {
        let result = bincode::deserialize::<(u128, DebugServerMsg)>(bytes);
        trace!("debug server msg deserialized from {:?} to {:?}", bytes, &result);
        result
    }

    pub fn as_bytes(&self, id: u128) -> Result<Vec<u8>, bincode::Error> {
        let object = (id, self.clone());
        let result = bincode::serialize::<(u128, DebugServerMsg)>(&object);
        trace!("debug server msg serialized from {:?} {:?}", &object, &result);
        result
    }
}