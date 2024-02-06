use crate::{database::models::user::User, server::client_msg::ClientMsg};
use crate::message::server_msg_img::AggImg;
use crate::server::registration_invalid::RegistrationInvalidMsg;
use serde::{Deserialize, Serialize};
// use rkyv::ser::serializers::{
//     AllocScratchError, CompositeSerializerError, SharedSerializeMapError,
// };
// use rkyv::{AlignedVec, Archive, Deserialize, Infallible, Serialize};
use thiserror::Error;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
// #[archive(compare(PartialEq), check_bytes)]
// #[archive_attr(derive(Debug))]
pub enum ServerMsg {
    Imgs(Vec<AggImg>),
    ProfileImgs(Option<Vec<AggImg>>),
    Profile(Option<User>),
    RegistrationInvalid(RegistrationInvalidMsg),
    RegistrationCompleted,
    LoginInvalid(String),
    LoginComplete { user_id: String, token: String },
    LoginFromTokenComplete { user_id: String },
    Ping,
    LoggedOut,
    None,
    Reset,
}

pub const SERVER_MSG_IMGS_NAME: &str = "imgs";
pub const SERVER_MSG_PROFILE_IMGS_NAME: &str = "profile_imgs";
pub const SERVER_MSG_PROFILE: &str = "profile";
pub const SERVER_MSG_REGISTRATION: &str = "registration";
pub const SERVER_MSG_LOGIN: &str = "login";
pub const SERVER_MSG_PING: &str = "ping";
pub const SERVER_MSG_RESET_NAME: &str = "reset";
pub const SERVER_MSG_NONE_NAME: &str = "NONE";

impl ServerMsg {
    pub fn name(&self) -> &'static str {
        match self {
            ServerMsg::Imgs(_a) => SERVER_MSG_IMGS_NAME,
            ServerMsg::ProfileImgs(_a) => SERVER_MSG_PROFILE_IMGS_NAME,
            ServerMsg::Profile(_) => SERVER_MSG_PROFILE,
            ServerMsg::RegistrationInvalid(_) => SERVER_MSG_REGISTRATION,
            ServerMsg::RegistrationCompleted => SERVER_MSG_REGISTRATION,
            ServerMsg::LoginInvalid(_) => SERVER_MSG_LOGIN,
            ServerMsg::LoginComplete { token, user_id } => SERVER_MSG_LOGIN,
            ServerMsg::LoginFromTokenComplete { user_id } => SERVER_MSG_LOGIN,
            ServerMsg::LoggedOut => SERVER_MSG_LOGIN,
            ServerMsg::Ping => SERVER_MSG_PING,
            ServerMsg::Reset => SERVER_MSG_RESET_NAME,
            ServerMsg::None => SERVER_MSG_NONE_NAME,
        }
    }
}

impl ServerMsg {
    pub fn from_bytes(bytes: &[u8]) -> Result<(u128, Self), bincode::Error> {
        bincode::deserialize::<(u128, ServerMsg)>(bytes)
    }

    pub fn as_bytes(&self, id: u128) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize::<(u128, ServerMsg)>(&(id, self.clone()))
    }
}
