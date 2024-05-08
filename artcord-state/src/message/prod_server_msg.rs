use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use crate::{
    aggregation::server_msg_img::AggImg,
    misc::registration_invalid::RegistrationInvalidMsg,
    model::{user::User, ws_statistics::{TempConIdType, WsStatDb, WsStatTemp}},
};

use artcord_leptos_web_sockets::WsPackage;
use serde::{Deserialize, Serialize};
use tracing::error;

use super::prod_client_msg::ClientMsgIndexType;


#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum ServerMsg {
    WsLiveStatsStarted(HashMap<TempConIdType, WsStatTemp>),
    WsLiveStatsUpdateRemoveStat { con_key: TempConIdType },
    WsLiveStatsUpdateAddedStat { con_key: TempConIdType, stat: WsStatTemp },
    WsLiveStatsUpdateInc { con_key: TempConIdType, path: ClientMsgIndexType },
    WsLiveStatsStopped,
    WsLiveStatsAlreadyStarted,
    WsLiveStatsAlreadyStopped,
    WsLiveStatsTaskIsNotSet,
    WsStatsTotalCount(u64),
    //WsStatsFirstPage { total_count: u64, first_page: Vec<WsStat> },
    WsStatsWithPagination { total_count: u64, latest: Option<i64>, stats: Vec<WsStatDb> },
    WsStatsPage(Vec<WsStatDb>),
    WsStatsGraph(Vec<f64>),
    GalleryMain(Vec<AggImg>),
    GalleryUser(Option<Vec<AggImg>>),
    User(Option<User>),
    LoginSuccess { user_id: String, token: String },
    LoginErr(String),
    RegistrationSuccess,
    RegistrationErr(RegistrationInvalidMsg),
    LoggedOut,

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

// #[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
// pub enum RegistrationRes {
//     Success,
//     Err(RegistrationInvalidMsg),
// }









impl artcord_leptos_web_sockets::Receive for ServerMsg {
    fn recv_from_vec(bytes: &[u8]) -> Result<WsPackage<Self>, String>
    where
        Self: std::marker::Sized + Clone,
    {
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
    pub fn from_bytes(bytes: &[u8]) -> Result<WsPackage<Self>, bincode::Error> {
        bincode::deserialize::<WsPackage<Self>>(bytes)
    }

    pub fn as_bytes(package: WsPackage<Self>) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize::<WsPackage<Self>>(&package)
    }
}
