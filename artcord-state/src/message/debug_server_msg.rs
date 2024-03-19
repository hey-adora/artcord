use artcord_leptos_web_sockets::WsRouteKey;
use tracing::{error, info, trace, warn};
use serde::{Deserialize, Serialize};

use super::debug_msg_key::DebugMsgPermKey;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub enum DebugServerMsg {
    Restart,
}

impl artcord_leptos_web_sockets::Receive<u128, DebugMsgPermKey> for DebugServerMsg {
    fn recv_from_vec(
            bytes: &[u8],
        ) -> Result<artcord_leptos_web_sockets::WsPackage<u128, DebugMsgPermKey, Self>, String>
        where
            Self: std::marker::Sized + Clone {
                DebugServerMsg::from_bytes(bytes).map_err(|e| e.to_string())
    }
}

impl DebugServerMsg {
    pub fn from_bytes(bytes: &[u8]) -> Result<artcord_leptos_web_sockets::WsPackage<u128, DebugMsgPermKey, Self>, bincode::Error> {
        let result = bincode::deserialize::<artcord_leptos_web_sockets::WsPackage<u128, DebugMsgPermKey, Self>>(bytes);
        trace!("debug server msg deserialized from {:?} to {:?}", bytes, &result);
        result
    }

    pub fn as_bytes(package: &artcord_leptos_web_sockets::WsPackage<u128, DebugMsgPermKey, Self>) -> Result<Vec<u8>, bincode::Error> {
        //let object = (id.clone(), *self);
        let result = bincode::serialize::<artcord_leptos_web_sockets::WsPackage<u128, DebugMsgPermKey, Self>>(package);
        trace!("debug server msg serialized from {:?} {:?}", &package, &result);
        result
    }
}