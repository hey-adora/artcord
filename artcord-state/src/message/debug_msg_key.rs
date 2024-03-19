use artcord_leptos_web_sockets::KeyGen;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub enum DebugMsgPermKey {
    Restart,
}

// impl KeyGen for DebugMsgKey {
//     fn generate_key() -> Self {
//         DebugMsgKey::Unique(uuid::Uuid::new_v4().as_u128())
//     }
// }