use artcord_leptos_web_sockets::WsRouteKey;
use serde::{Deserialize, Serialize};
use crate::{aggregation::server_msg_img::AggImg, misc::registration_invalid::RegistrationInvalidMsg, model::user::User};

use super::prod_perm_key::ProdMsgPermKey;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum ServerMsg {
    MainGallery(MainGalleryResponse),

    // Imgs(Vec<AggImg>),
    // ProfileImgs(Option<Vec<AggImg>>),
    // Profile(Option<User>),
    // RegistrationInvalid(RegistrationInvalidMsg),
    // RegistrationCompleted,
    // LoginInvalid(String),
    // LoginComplete { user_id: String, token: String },
    // LoginFromTokenComplete { user_id: String },
    // Ping,
    // LoggedOut,
    Error,
    None,
    Reset,
    NotImplemented,
    // Error(String),
}

// #[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
// pub enum ServerError {
//     DatabaseError,
//     Uknown
// }

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum MainGalleryResponse {
    Imgs(Vec<AggImg>),

}

impl artcord_leptos_web_sockets::Receive<u128, ProdMsgPermKey> for ServerMsg {
    fn recv_from_vec(
            bytes: &[u8],
        ) -> Result<artcord_leptos_web_sockets::WsPackage<u128, ProdMsgPermKey, Self>, String>
        where
            Self: std::marker::Sized + Clone {
                ServerMsg::from_bytes(bytes).map_err(|e| e.to_string())
    }
}

// pub const SERVER_MSG_IMGS_NAME: &str = "imgs";
// pub const SERVER_MSG_PROFILE_IMGS_NAME: &str = "profile_imgs";
// pub const SERVER_MSG_PROFILE: &str = "profile";
// pub const SERVER_MSG_REGISTRATION: &str = "registration";
// pub const SERVER_MSG_LOGIN: &str = "login";
// pub const SERVER_MSG_PING: &str = "ping";
// pub const SERVER_MSG_RESET_NAME: &str = "reset";
// pub const SERVER_MSG_NONE_NAME: &str = "NONE";

// impl ServerMsg {
//     pub fn name(&self) -> &'static str {
//         match self {
//             ServerMsg::Imgs(_a) => SERVER_MSG_IMGS_NAME,
//             ServerMsg::ProfileImgs(_a) => SERVER_MSG_PROFILE_IMGS_NAME,
//             ServerMsg::Profile(_) => SERVER_MSG_PROFILE,
//             ServerMsg::RegistrationInvalid(_) => SERVER_MSG_REGISTRATION,
//             ServerMsg::RegistrationCompleted => SERVER_MSG_REGISTRATION,
//             ServerMsg::LoginInvalid(_) => SERVER_MSG_LOGIN,
//             ServerMsg::LoginComplete { token: _, user_id: _ } => SERVER_MSG_LOGIN,
//             ServerMsg::LoginFromTokenComplete { user_id: _ } => SERVER_MSG_LOGIN,
//             ServerMsg::LoggedOut => SERVER_MSG_LOGIN,
//             ServerMsg::Ping => SERVER_MSG_PING,
//             ServerMsg::Reset => SERVER_MSG_RESET_NAME,
//             ServerMsg::None => SERVER_MSG_NONE_NAME,
//         }
//     }
// }

impl ServerMsg {
    pub fn from_bytes(bytes: &[u8]) -> Result<artcord_leptos_web_sockets::WsPackage<u128, ProdMsgPermKey, Self>, bincode::Error> {
        bincode::deserialize::<artcord_leptos_web_sockets::WsPackage<u128, ProdMsgPermKey, Self>>(bytes)
    }

    pub fn as_bytes(package: artcord_leptos_web_sockets::WsPackage<u128, ProdMsgPermKey, Self>) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize::<artcord_leptos_web_sockets::WsPackage<u128, ProdMsgPermKey, Self>>(&package)
    }
}
