use artcord_leptos_web_sockets::WsRouteKey;
use serde::{Deserialize, Serialize};
use tracing::{error, info, trace, warn};

use super::debug_msg_key::DebugMsgPermKey;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub enum DebugClientMsg {
    BrowserReady,
    RuntimeReady,
}

impl artcord_leptos_web_sockets::Send for DebugClientMsg {
    fn send_as_vec(package: &artcord_leptos_web_sockets::WsPackage<Self>) -> Result<Vec<u8>, String>
    where
        Self: Clone,
    {
        Self::as_vec(package).map_err(|e| e.to_string())
    }
}

impl DebugClientMsg {
    pub fn as_vec(
        package: &artcord_leptos_web_sockets::WsPackage<Self>,
    ) -> Result<Vec<u8>, bincode::Error> {
        //let object = (id.clone(), *self);
        let result: Result<Vec<u8>, Box<bincode::ErrorKind>> =
            bincode::serialize::<artcord_leptos_web_sockets::WsPackage<Self>>(package);
        trace!(
            "debug client msg serialized from {:?} {:?}",
            package,
            &result
        );
        result
    }

    pub fn from_bytes(
        bytes: &[u8],
    ) -> Result<artcord_leptos_web_sockets::WsPackage<Self>, bincode::Error> {
        let result: Result<artcord_leptos_web_sockets::WsPackage<Self>, Box<bincode::ErrorKind>> =
            bincode::deserialize::<artcord_leptos_web_sockets::WsPackage<Self>>(bytes);
        trace!(
            "debug client msg deserialized from {:?} to {:?}",
            bytes,
            &result
        );
        result
    }
}

