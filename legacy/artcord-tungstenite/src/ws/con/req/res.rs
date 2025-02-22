use artcord_mongodb::database::DBError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::Message;
use artcord_state::{global, backend};



pub mod ws_stats;
pub mod ws_throttle;
pub mod user;
pub mod auth;
pub mod gallery;

#[derive(Error, Debug)]
pub enum ResErr {
    // global::WsStatDbToSavedErr
    #[error("failed to convert ws_con: {0}")]
    WsConFromDbErr(#[from] global::WsConFromDbErr),

    #[error("Invalid client msg type (not binary) error: {0}")]
    InvalidClientMsg(Message),

    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("oneshot recv error: {0}")]
    OneShotRecv(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("Send error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),

    #[error("Send error: {0}")]
    WsAppSend(#[from] tokio::sync::mpsc::error::SendError<backend::WsMsg>),

    #[error("Send error: {0}")]
    ConnectionSend(#[from] tokio::sync::mpsc::error::SendError<backend::ConMsg>),

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

    #[error("Jwt error: {0}")]
    JwtErr(#[from] jsonwebtoken::errors::Error),
}