use crate::database::models::user::User;
use crate::server::registration_invalid::RegistrationInvalidMsg;
use crate::server::server_msg_img::ServerMsgImg;
use rkyv::ser::serializers::{
    AllocScratchError, CompositeSerializerError, SharedSerializeMapError,
};
use rkyv::{AlignedVec, Archive, Deserialize, Infallible, Serialize};
use thiserror::Error;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub enum ServerMsg {
    Imgs(Vec<ServerMsgImg>),
    ProfileImgs(Option<Vec<ServerMsgImg>>),
    Profile(Option<User>),
    RegistrationInvalid(RegistrationInvalidMsg),
    RegistrationCompleted,
    LoginInvalid(String),
    LoginComplete(String),
    LoginFromTokenComplete,
    LoggedOut,
    None,
    Reset,
}

pub const SERVER_MSG_IMGS_NAME: &str = "imgs";
pub const SERVER_MSG_PROFILE_IMGS_NAME: &str = "profile_imgs";
pub const SERVER_MSG_PROFILE: &str = "profile";
pub const SERVER_MSG_REGISTRATION: &str = "registration";
pub const SERVER_MSG_LOGIN: &str = "login";
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
            ServerMsg::LoginComplete(_) => SERVER_MSG_LOGIN,
            ServerMsg::LoginFromTokenComplete => SERVER_MSG_LOGIN,
            ServerMsg::LoggedOut => SERVER_MSG_LOGIN,
            ServerMsg::Reset => SERVER_MSG_RESET_NAME,
            ServerMsg::None => SERVER_MSG_NONE_NAME,
        }
    }
}

#[derive(Error, Debug)]
pub enum WebSerializeError {
    #[error("Invalid bytes, error: {0}")]
    InvalidBytes(String),
}

impl ServerMsg {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WebSerializeError> {
        let server_msg: Self = rkyv::check_archived_root::<Self>(bytes)
            .or_else(|e| {
                Err(WebSerializeError::InvalidBytes(format!(
                    "Received invalid binary msg: {}",
                    e
                )))
            })?
            .deserialize(&mut rkyv::Infallible)
            .or_else(|e| {
                Err(WebSerializeError::InvalidBytes(format!(
                    "Received invalid binary msg: {:?}",
                    e
                )))
            })?;

        Ok(server_msg)
    }

    pub fn as_bytes(
        &self,
    ) -> Result<
        AlignedVec,
        CompositeSerializerError<
            std::convert::Infallible,
            AllocScratchError,
            SharedSerializeMapError,
        >,
    > {
        rkyv::to_bytes::<_, 256>(self)
    }
}
