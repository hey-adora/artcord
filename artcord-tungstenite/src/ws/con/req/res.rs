use artcord_mongodb::database::DBError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::Message;

use crate::ws::{con::ConMsg, WsAppMsg};

pub mod ws_stats;
pub mod ws_throttle;
pub mod user;
pub mod auth;
pub mod gallery;

#[derive(Error, Debug)]
pub enum ResErr {
    #[error("Invalid client msg type (not binary) error: {0}")]
    InvalidClientMsg(Message),

    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("oneshot recv error: {0}")]
    OneShotRecv(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("Send error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),

    #[error("Send error: {0}")]
    WsAppSend(#[from] tokio::sync::mpsc::error::SendError<WsAppMsg>),

    #[error("Send error: {0}")]
    ConnectionSend(#[from] tokio::sync::mpsc::error::SendError<ConMsg>),

    // #[error("Send error: {0}")]
    // ThrottleSend(#[from] tokio::sync::mpsc::error::SendError<WsStatsMsg>),

    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),

    #[error("DB Error error: {0}")]
    DBError(#[from] DBError),

    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("RwLock error: {0}")]
    RwLock(String),
}