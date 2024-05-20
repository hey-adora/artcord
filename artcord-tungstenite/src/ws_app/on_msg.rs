use crate::ws_app::ws_throttle::WsThrottle;
use crate::ws_app::WsAppMsg;
use artcord_leptos_web_sockets::WsPackage;
use artcord_state::message::prod_server_msg::ServerMsg;
use chrono::DateTime;
use chrono::Utc;
use tokio_tungstenite::tungstenite::Message;
use tracing::trace;
use tracing::debug;
use thiserror::Error;

use super::on_connection::con_task::ConMsg;

pub async fn on_ws_msg(msg: Option<WsAppMsg>, throttle: &mut WsThrottle, time: DateTime<Utc>) -> Result<bool, WsMsgErr> {
    let Some(msg) = msg else {
        trace!("ws_recv channel closed");
        return Ok(true);
    };
    match msg {
        WsAppMsg::Inc { ip, path } => {
            throttle.on_inc(ip, path, time).await?;
        }
        WsAppMsg::Disconnected { connection_key, ip } => {
            throttle.on_disconnected(ip, connection_key).await?;
        }
        WsAppMsg::Stop => {
            return Ok(true);
        }
        WsAppMsg::AddListener { connection_key, tx, ws_key } => {
            throttle.add_listener(connection_key, ws_key, tx).await?;
        }
        WsAppMsg::RemoveListener { connection_key, tx, ws_key} => {
            throttle.remove_listener(connection_key, ws_key, tx).await?;
        }
        WsAppMsg::Ban { ip, until, reason } => {
            throttle.on_ban(&ip, reason, until).await?;
        }
    }
    Ok(false)
}

#[derive(Error, Debug)]
pub enum WsMsgErr {
    #[error("MainGallery error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Send error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),

    #[error("Send error: {0}")]
    ConnectionSend(#[from] tokio::sync::mpsc::error::SendError<ConMsg>),

    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),
}
