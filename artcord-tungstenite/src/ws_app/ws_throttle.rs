use std::collections::HashMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use artcord_leptos_web_sockets::WsPackage;
use artcord_leptos_web_sockets::WsRouteKey;
use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::util::time::time_is_past;
use artcord_state::util::time::time_passed_days;
use chrono::DateTime;
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

use crate::WS_BAN_UNTIL_DAYS;
use crate::WS_EXPIRE_RED_FLAGS_DAYS;
use crate::WS_LIMIT_MAX_CONNECTIONS;
use crate::WS_LIMIT_MAX_RED_FLAGS;

use super::on_connection::con_task::ConMsg;

pub struct ThrottleStats {
    ws_connection_count: u64,
    // wrap hashmap in SocketAddr (maybe)
    ws_path_count: HashMap<WsPath, (u64, Instant)>,
    ws_red_flag: Option<(u64, DateTime<Utc>)>,
    ws_banned_until: Option<DateTime<Utc>>,
    // ws_proccesing: RwLock<bool>,
    // ws_path_interval: RwLock<DateTime<chrono::Utc>>,
    //ws_last_connection: RwLock<u64>,
}

impl ThrottleStats {
    pub fn new() -> Self {
        Self {
            ws_connection_count: 1,
            ws_path_count: HashMap::new(),
            ws_red_flag: None,
            ws_banned_until: None,
            //   ws_path_interval: RwLock::new(Utc::now())
        }
    }

    // pub async fn maybe_sleep(&self, ws_path: &WsPath) {
    //     let mut ws_path_count_guard = self.ws_path_count.write().await;
    //     // let (count, interval) = ws_path_count.entry(ws_path).or_insert((1, Instant::now()));
    //     let ws_path_count = ws_path_count_guard.get_mut(ws_path);
    //     if let Some((count, interval)) = ws_path_count {
    //         let (count_limit, interval_limit) = ws_path.get_throttle();
    //         let elapsed = interval.elapsed();
    //         if elapsed > interval_limit {
    //             trace!("throttle: reset");
    //             *count = 0;
    //             *interval = Instant::now();
    //         } else if *count > count_limit {
    //             let left = interval_limit.checked_sub(elapsed);
    //             if let Some(left) = left {
    //                 trace!("throttle: sleeping for: {:?}", &left);
    //                 sleep(left).await;
    //             } else {
    //                 error!("throttle: failed to get left time");
    //                 sleep(interval_limit).await;
    //             }
    //             *count = 0;
    //             *interval = Instant::now();
    //             trace!("throttle: sleep completed");
    //         } else {
    //             trace!("throttle: all good: state: {} {:?}", &count, &elapsed);
    //             *count += 1;
    //         }
    //     } else {
    //         let new_ws_path_count = (1_u64, Instant::now());
    //         ws_path_count_guard.insert(ws_path.clone(), new_ws_path_count);
    //     }
    // }
    //
    // // pub async fn maybe_connect_to_ws() {
    // //         if let Some(user_throttle_stats) = user_throttle_stats {
    // //             trace!("ws({}): throttle: stats exist", &ws_addr);
    // //             let count = *user_throttle_stats.ws_connection_count.read().await;
    // //
    // //             // let (time, count) = *throttle.read().await;
    // //
    // //             // let throttle = match throttle {
    // //             //     Ok(result) => result,
    // //             //     Err(err) => {
    // //             //         error!("ws({}): lock error: {}", &ws_addr, err);
    // //             //         continue;
    // //             //     }
    // //             // };
    // //
    // //             // (time, count)
    // //             trace!(
    // //                 "ws({}): throttle: {} > {}",
    // //                 &ws_addr,
    // //                 count,
    // //                 WS_LIMIT_MAX_CONNECTIONS
    // //             );
    // //             if count > WS_LIMIT_MAX_CONNECTIONS {
    // //                 trace!("ws({}): connection limit reached: {}", &ws_addr, count);
    // //                 continue;
    // //             }
    // //             *user_throttle_stats.ws_connection_count.write().await += 1;
    // //             trace!(
    // //                 "ws({}): throttle: incremented to: {}",
    // //                 &ws_addr,
    // //                 *user_throttle_stats.ws_connection_count.read().await
    // //             );
    // //             user_throttle_stats.clone()
    // //         } else {
    // //             trace!("ws({}): throttle: created new", &ws_addr);
    // //             let user_throttle_stats = Arc::new(ThrottleStats::new());
    // //             throttle.insert(ip, user_throttle_stats.clone());
    // //             user_throttle_stats
    // //         }
    // // }
    //
    pub async fn is_banned(&self) -> bool {
        let is_baned = self
            .ws_banned_until
            .map(|until| !time_is_past(until))
            .unwrap_or_else(|| {
                trace!("throttle: ban check: entry doesnt exist");
                false
            });
        trace!(
            "throttle: is banned: {}, state: {:#?}",
            is_baned,
            self.ws_banned_until
        );

        is_baned
    }
    //
    // pub async fn maybe_ban(&self) {
    //     let red_flag = *self.ws_red_flag.read().await;
    //     if let Some((count, last_modified)) = red_flag {
    //         if time_passed_days(last_modified, WS_EXPIRE_RED_FLAGS_DAYS) {
    //             let red_flag = &mut *self.ws_red_flag.write().await;
    //             trace!("throttle: ws_red_flag: {:?} to None", red_flag,);
    //             *red_flag = None;
    //         } else if count > WS_LIMIT_MAX_RED_FLAGS {
    //             let now = Utc::now();
    //             let ban = self
    //                 .ws_banned_until
    //                 .read()
    //                 .await
    //                 .clone()
    //                 .map(|until| now > until)
    //                 .unwrap_or(true);
    //
    //             if ban {
    //                 let new_date = now + chrono::Days::new(WS_BAN_UNTIL_DAYS);
    //                 trace!("throttle: banned until: {}", &new_date,);
    //
    //                 *self.ws_banned_until.write().await = Some(new_date);
    //                 debug!("IM HEREEEEEEEEEEEEEEEEEEEEEEEEEEEEEE");
    //             } else {
    //                 trace!("throttle: is already banned");
    //             }
    //             // if let Some(banned_until) = banned_until {
    //             //     // banned_until.
    //             // } else {
    //             //     *banned_until = Some(Utc::now() + Months::new(1));
    //             // }
    //         } else {
    //             let red_flag = &mut *self.ws_red_flag.write().await;
    //             if let Some((count, last_modified)) = red_flag {
    //                 let new_date = Utc::now();
    //                 trace!(
    //                     "throttle: ws_red_flag: ({}, {}) to ({}, {})",
    //                     count,
    //                     last_modified,
    //                     *count + 1,
    //                     new_date
    //                 );
    //                 *count += 1;
    //                 *last_modified = new_date;
    //             } else {
    //                 error!("throttle: failed to get ws_red_flag");
    //             }
    //         }
    //     } else {
    //         let new_red_flag = Some((1, Utc::now()));
    //         trace!("throttle: new ws_red_flag created: {:?}", &new_red_flag);
    //         *self.ws_red_flag.write().await = new_red_flag;
    //     }
    // }
}

pub struct WsThrottle {
    pub ips: HashMap<IpAddr, ThrottleStats>,
}

impl WsThrottle {
    pub fn new() -> Self {
        Self {
            ips: HashMap::new(),
        }
    }

    pub async fn disconnected(&mut self, ip: &IpAddr) {
        let ip_stats = self.ips.get_mut(ip);
        let Some(ip_stats) = ip_stats else {
            error!("throttle: cant disconnect ip that doesnt exist");
            return;
        };
        let val = ip_stats.ws_connection_count.checked_sub(1);
        let Some(val) = val else {
            error!("throttle: overflow prevented");
            return;
        };

        ip_stats.ws_connection_count = val;
    }

    pub async fn is_bad(&mut self, ip: IpAddr) -> bool {
        let Some(user_throttle_stats) = self.ips.get_mut(&ip) else {
            trace!("ws({}): throttle: created new", &ip);
            self.ips.insert(ip, ThrottleStats::new());
            return false;
        };
        if user_throttle_stats.is_banned().await {
            trace!("ws({}): throttle: is banned", &ip);
            return true;
        }
        trace!("ws({}): throttle: stats exist", &ip);
        // let count = user_throttle_stats.ws_connection_count;

        // let (time, count) = *throttle.read().await;

        // let throttle = match throttle {
        //     Ok(result) => result,
        //     Err(err) => {
        //         error!("ws({}): lock error: {}", &ws_addr, err);
        //         continue;
        //     }
        // };

        // (time, count)
        trace!(
            "ws({}): throttle: {} > {}",
            &ip,
            user_throttle_stats.ws_connection_count,
            WS_LIMIT_MAX_CONNECTIONS
        );
        if user_throttle_stats.ws_connection_count > WS_LIMIT_MAX_CONNECTIONS {
            trace!(
                "ws({}): connection limit reached: {}",
                &ip,
                user_throttle_stats.ws_connection_count
            );
            return true;
        }
        user_throttle_stats.ws_connection_count += 1;
        trace!(
            "ws({}): throttle: incremented to: {}",
            &ip,
            user_throttle_stats.ws_connection_count
        );
        false
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
    // #[error("Mongodb error: {0}")]
    // MongoDB(#[from] mongodb::error::Error),

    // #[error("Bcrypt error: {0}")]
    // Bcrypt(#[from] bcrypt::BcryptError),
    //
    // #[error("JWT error: {0}")]
    // JWT(#[from] jsonwebtoken::errors::Error),
    // #[error("RwLock error: {0}")]
    // RwLock(String),
}