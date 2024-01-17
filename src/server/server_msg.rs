use crate::database::models::user::User;
use crate::server::server_msg_img::ServerMsgImg;
use rkyv::{Archive, Deserialize, Serialize};
use thiserror::Error;
use crate::server::registration_invalid::RegistrationInvalidMsg;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub enum ServerMsg {
    Imgs(Vec<ServerMsgImg>),
    ProfileImgs(Option<Vec<ServerMsgImg>>),
    Profile(Option<User>),
    RegistrationInvalid(RegistrationInvalidMsg),
    RegistrationCompleted,
    None,
    Reset,
}

pub const SERVER_MSG_IMGS_NAME: &str = "imgs";
pub const SERVER_MSG_PROFILE_IMGS_NAME: &str = "profile_imgs";
pub const SERVER_MSG_PROFILE: &str = "profile";
pub const SERVER_MSG_RESET_NAME: &str = "reset";
pub const SERVER_MSG_REGISTRATION_COMPLETED: &str = "registration_completed";
pub const SERVER_MSG_REGISTRATION_INVALID: &str = "registration_invalid";
pub const SERVER_MSG_NONE_NAME: &str = "NONE";

impl ServerMsg {
    pub fn name(&self) -> String {
        match self {
            ServerMsg::Imgs(_a) => String::from(SERVER_MSG_IMGS_NAME),
            ServerMsg::ProfileImgs(_a) => String::from(SERVER_MSG_PROFILE_IMGS_NAME),
            ServerMsg::Profile(_) => String::from(SERVER_MSG_PROFILE),
            ServerMsg::RegistrationInvalid(_) => String::from(SERVER_MSG_REGISTRATION_INVALID),
            ServerMsg::RegistrationCompleted => String::from(SERVER_MSG_REGISTRATION_COMPLETED),
            ServerMsg::Reset => String::from(SERVER_MSG_RESET_NAME),
            ServerMsg::None => String::from(SERVER_MSG_NONE_NAME),
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
}
