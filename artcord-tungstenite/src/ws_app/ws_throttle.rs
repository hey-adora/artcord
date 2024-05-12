use std::collections::HashMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::ops::Div;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use artcord_leptos_web_sockets::WsPackage;
use artcord_leptos_web_sockets::WsRouteKey;
use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::ClientMsgIndexType;
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::misc::throttle_connection::ConStatus;
use artcord_state::misc::throttle_connection::IpBanReason;
use artcord_state::misc::throttle_connection::LiveThrottleConnection;
use artcord_state::misc::throttle_threshold::AllowCon;
use artcord_state::model::ws_statistics::TempConIdType;
use artcord_state::util::time::time_is_past;
use artcord_state::util::time::time_passed_days;
use chrono::DateTime;
use chrono::Days;
use chrono::Month;
use chrono::Months;
use chrono::TimeDelta;
use chrono::Utc;
use futures::pin_mut;
use futures::FutureExt;
use futures::TryStreamExt;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::task;

use cfg_if::cfg_if;
use futures::future;
use futures::SinkExt;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time;
use tokio::time::sleep;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::debug;
use tracing::instrument;
use tracing::Instrument;
use tracing::{error, trace};

use crate::WS_CON_THRESHOLD;
use crate::WS_CON_THRESHOLD_BAN_DURATION;
//use crate::WS_MAX_FAILED_CON_ATTEMPTS;
use crate::WS_BAN_UNTIL_DAYS;
use crate::WS_EXPIRE_RED_FLAGS_DAYS;
use crate::WS_CON_THRESHOLD_RANGE;
use crate::WS_LIMIT_MAX_RED_FLAGS;
//use crate::WS_MAX_FAILED_CON_ATTEMPTS_DELTA;
//use crate::WS_MAX_FAILED_CON_ATTEMPTS_RATE;

use super::on_connection::con_task::ConMsg;
use super::on_msg::WsMsgErr;





pub struct WsThrottle {
    pub ips: HashMap<IpAddr, LiveThrottleConnection>,
    pub listener_list: HashMap<TempConIdType, (WsRouteKey, mpsc::Sender<ConMsg>)>,
}

impl WsThrottle {
    pub fn new() -> Self {
        Self {
            ips: HashMap::new(),
            listener_list: HashMap::new(),
        }
    }

    // pub async fn send_update_to_listeners(&self) {
    //     let reached_max = user_throttle_stats.reached_max_con(&ip, WS_LIMIT_MAX_CONNECTIONS, WS_MAX_FAILED_CON_ATTEMPTS, WS_MAX_FAILED_CON_ATTEMPTS_RATE, WS_BAN_UNTIL_DAYS);
    //     if !reached_max {
    //         let msg = ServerMsg::WsLiveThrottleCachedConnected { ip } ;
    //         let mut to_remove: Vec<TempConIdType> = Vec::new();
    //         for (con_key, (ws_key, tx)) in self.listener_list.iter() {
    //             let msg: WsPackage<ServerMsg> = (ws_key.clone(), msg.clone());
    //             let msg = ServerMsg::as_bytes(msg)?;
    //             let msg = Message::binary(msg);
    //             let send_result = tx.send(ConMsg::Send(msg)).await;
    //             if let Err(err) = send_result {
    //                 debug!("ws({}): throttle: failed to send on_con update to {} {}", &ip, con_key, err);
    //                 to_remove.push(*con_key);
    //             }
    //         }
    //         for con_key in to_remove {
    //             self.listener_list.remove(&con_key);
    //         }
    //     }
    // }

    pub async fn send_update_to_listeners(&mut self, ip: &IpAddr, msg: ServerMsg) -> Result<(), WsMsgErr> {
        let mut to_remove: Vec<TempConIdType> = Vec::new();
        for (con_key, (ws_key, tx)) in self.listener_list.iter() {
            let msg: WsPackage<ServerMsg> = (ws_key.clone(), msg.clone());
            let msg = ServerMsg::as_bytes(msg)?;
            let msg = Message::binary(msg);
            let send_result = tx.send(ConMsg::Send(msg)).await;
            if let Err(err) = send_result {
                debug!("ws({}): throttle: failed to send on_con update to {} {}", ip, con_key, err);
                to_remove.push(*con_key);
            }
        }
        for con_key in to_remove {
            self.listener_list.remove(&con_key);
        }
        Ok(())
    }

    pub async fn add_listener(&mut self, con_key: TempConIdType, ws_key: WsRouteKey, tx: mpsc::Sender<ConMsg>) -> Result<bool, WsMsgErr> {
        trace!("ws_app: listener added: {}", con_key);

        let msg = if self.listener_list.insert(con_key, (ws_key, tx.clone())).is_some() {
            ServerMsg::WsLiveThrottleCachedEntryUpdated(self.ips.clone())
        } else {
            ServerMsg::WsLiveThrottleCachedEntryAdded(self.ips.clone())
        };
        let msg: WsPackage<ServerMsg> = (ws_key.clone(), msg);
        let msg = ServerMsg::as_bytes(msg)?;
        let msg = Message::binary(msg);
        tx.send(ConMsg::Send(msg)).await?;

        Ok(false)
    }

    pub async fn remove_listener(&mut self, con_key: TempConIdType, ws_key: WsRouteKey, tx: mpsc::Sender<ConMsg>) -> Result<bool, WsMsgErr> {
        trace!("ws_app: listener removed: {}", con_key);
        let Some((ws_key, tx)) = self.listener_list.remove(&con_key) else {
            debug!("ws_app: listener not found: {}", con_key);
            let msg = ServerMsg::WsLiveThrottleCachedEntryNotFound;
            let msg: WsPackage<ServerMsg> = (ws_key.clone(), msg);
            let msg = ServerMsg::as_bytes(msg)?;
            let msg = Message::binary(msg);
            tx.send(ConMsg::Send(msg)).await?;
            return Ok(false);
        };
        let msg = ServerMsg::WsLivThrottleCachedEntryRemoved;
        let msg: WsPackage<ServerMsg> = (ws_key.clone(), msg);
        let msg = ServerMsg::as_bytes(msg)?;
        let msg = Message::binary(msg);
        tx.send(ConMsg::Send(msg)).await?;

        Ok(false)
    }

    pub async fn on_inc(&mut self, ip: IpAddr, path: ClientMsgIndexType) -> Result<bool, WsMsgErr> {
        let con = self.ips.get_mut(&ip);
        let Some(con) = con else {
            return Ok(false);
        };
        con.inc_path(&path);

        let msg = ServerMsg::WsLiveThrottleCachedIncPath { ip, path };
        for (_con_key, (ws_key, tx)) in self.listener_list.iter() {
            let msg: WsPackage<ServerMsg> = (ws_key.clone(), msg.clone());
            let msg = ServerMsg::as_bytes(msg)?;
            let msg = Message::binary(msg);
            tx.send(ConMsg::Send(msg)).await?;
        }
        Ok(false)
    }

    pub async fn on_disconnected(&mut self, ip: IpAddr, con_id: TempConIdType) -> Result<bool, WsMsgErr>  {
        let ip_stats = self.ips.get_mut(&ip);
        let Some(ip_stats) = ip_stats else {
            error!("throttle: cant disconnect ip that doesnt exist");
            return Ok(false);
        };
        ip_stats.throttle.dec();
        if ip_stats.throttle.amount == 0 {
            self.listener_list.remove(&con_id);
        }
        
        let msg = ServerMsg::WsLiveThrottleCachedDisconnected { ip } ;
        for (_con_key, (ws_key, tx)) in self.listener_list.iter() {
            let msg: WsPackage<ServerMsg> = (ws_key.clone(), msg.clone());
            let msg = ServerMsg::as_bytes(msg)?;
            let msg = Message::binary(msg);
            tx.send(ConMsg::Send(msg)).await?;
        }

        Ok(false)
    }

    pub async fn on_connect(&mut self, ip: IpAddr, time: DateTime<Utc>) -> Result<bool, WsMsgErr> {
        let Some(con) = self.ips.get_mut(&ip) else {
            trace!("ws({}): throttle: created new", &ip);
            self.ips.insert(ip, LiveThrottleConnection::new(WS_CON_THRESHOLD_RANGE, time));
            return Ok(false);
        };

        let result = con.throttle.inc(&WS_CON_THRESHOLD, IpBanReason::TooManyConnectionAttempts, WS_CON_THRESHOLD_BAN_DURATION, time);

        match result {
            AllowCon::Allow => {
                let msg = ServerMsg::WsLiveThrottleCachedConnected { ip };
                self.send_update_to_listeners(&ip, msg).await?;
                Ok(false)
            }
            AllowCon::Blocked => {
                let msg = ServerMsg::WsLiveThrottleCachedBlocks { ip, total_blocks: con.throttle.tracker.total_amount, blocks: con.throttle.tracker.amount } ;
                self.send_update_to_listeners(&ip, msg).await?;
                Ok(true)
            }
            AllowCon::Banned((date, reason)) => {
                let msg = ServerMsg::WsLiveThrottleCachedBanned { ip, date, reason };
                self.send_update_to_listeners(&ip, msg).await?;
                Ok(true)
            }
            AllowCon::AlreadyBanned => {
                Ok(true)
            }
            AllowCon::Unbanned => {
                Ok(false)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum AdminMsgErr {
    // #[error("Statistics error: {0}")]
    // AdminThrottle(#[from] WsHandleAdminThrottleError),
    //
    // #[error("Statistics error: {0}")]
    // Statistics(#[from] WsStatisticsError),
    //
    // #[error("MainGallery error: {0}")]
    // MainGallery(#[from] WsHandleMainGalleryError),
    //
    // #[error("MainGallery error: {0}")]
    // UserGallery(#[from] WsHandleUserGalleryError),
    //
    // #[error("User error: {0}")]
    // User(#[from] WsHandleUserError),
    // #[error("Invalid client msg error")]
    // InvalidClientMsg,
    #[error("MainGallery error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Send error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),

    #[error("Send error: {0}")]
    ConnectionSend(#[from] tokio::sync::mpsc::error::SendError<ConMsg>),
    //
    // #[error("Send error: {0}")]
    // ThrottleSend(#[from] tokio::sync::mpsc::error::SendError<WsThrottleListenerMsg>),
    // // tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>>>
    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),
    // #[error("Bcrypt error: {0}")]
    // Bcrypt(#[from] bcrypt::BcryptError),
    //
    // #[error("JWT error: {0}")]
    // JWT(#[from] jsonwebtoken::errors::Error),
    // #[error("RwLock error: {0}")]
    // RwLock(String),
}
