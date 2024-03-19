use artcord_leptos_web_sockets::KeyGen;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub enum ProdMsgPermKey {
    Login,
    Reset
}