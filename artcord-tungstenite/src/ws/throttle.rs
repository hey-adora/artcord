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
use artcord_state::message::prod_client_msg::ClientPathType;
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::misc::throttle_connection::ConStatus;
use artcord_state::misc::throttle_connection::IpBanReason;
use artcord_state::misc::throttle_connection::LiveThrottleConnectionCount;
use artcord_state::misc::throttle_connection::TempThrottleConnection;
use artcord_state::misc::throttle_connection::WsReqStat;
use artcord_state::misc::throttle_threshold::AllowCon;
use artcord_state::misc::throttle_threshold::IsBanned;
use artcord_state::misc::throttle_threshold::Threshold;
use artcord_state::misc::throttle_threshold::ThrottleRanged;
use artcord_state::misc::throttle_threshold::ThrottleSimple;
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
use tokio::sync::watch;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::task;

use crate::WsThreshold;
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

use super::con::ConMsg;
use super::WsAppMsg;

#[derive(Debug, Clone)]
pub struct WsThrottle {
    pub ips: HashMap<IpAddr, WsThrottleCon>,
}

#[derive(Debug, Clone)]
pub struct WsThrottleCon {
    pub con_throttle: ThrottleRanged,
    pub con_flicker_throttle: ThrottleSimple,
    pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
}

impl WsThrottle {
    pub fn new() -> Self {
        Self {
            ips: HashMap::new(),
        }
    }
    pub fn on_ban(&mut self, ip: &IpAddr, ban_reason: IpBanReason, until: DateTime<Utc>) {
        let ip_stats = self.ips.get_mut(&ip);
        let Some(ip_stats) = ip_stats else {
            error!("throttle: cant be banned because it doesnt exist in the list");
            return;
        };
        ip_stats
            .con_throttle
            .ban(&mut ip_stats.banned_until, ban_reason, until);
    }

    pub fn dec_con(&mut self, ip: IpAddr, con_id: TempConIdType) {
        let ip_stats = self.ips.get_mut(&ip);
        let Some(ip_stats) = ip_stats else {
            error!("throttle: cant disconnect ip that doesnt exist");
            return;
        };
        ip_stats.dec();
    }

    pub fn get_amounts(&mut self, ip: IpAddr) -> Option<(u64, u64)> {
        let Some(con) = self.ips.get_mut(&ip) else {
            return None;
        };
        Some((con.con_throttle.tracker.total_amount, con.con_throttle.tracker.amount))
    }

    pub fn inc_con(
        &mut self,
        ip: IpAddr,
        ws_threshold: &WsThreshold,
        time: &DateTime<Utc>,
    ) -> AllowCon {
        let Some(con) = self.ips.get_mut(&ip) else {
            trace!("ws({}): throttle: created new", &ip);
            let con = WsThrottleCon::new(ws_threshold.ws_max_con_threshold_range, *time);
            self.ips.insert(ip, con);
            return AllowCon::Allow;
        };

        con.inc(ws_threshold, time)
    }
}

impl WsThrottleCon {
    pub fn to_temp(
        value: &HashMap<IpAddr, WsThrottleCon>,
    ) -> HashMap<IpAddr, TempThrottleConnection> {
        value
            .into_iter()
            .fold(HashMap::new(), |mut a, (key, value)| {
                a.insert(*key, value.into());
                a
            })
    }

    pub fn new(range: u64, started_at: DateTime<Utc>) -> Self {
        let con = Self {
            //path_stats: HashMap::new(),
            con_throttle: ThrottleRanged::new(range, started_at),
            con_flicker_throttle: ThrottleSimple::new(started_at),
            banned_until: None,
            // ip_stats_tx: ip_stats_tx.clone(),
            // ip_stats_rx: ip_stats_rx.clone(),
        };
        // ((ip_stats_tx, ip_stats_rx), con)
        con
    }

    pub fn dec(&mut self) {
        self.con_throttle.dec();
    }

    pub fn inc(&mut self, ws_threshold: &WsThreshold, time: &DateTime<Utc>) -> AllowCon {
        let allow = self.con_flicker_throttle.allow(
            &ws_threshold.ws_con_flicker_threshold,
            &ws_threshold.ws_con_flicker_ban_duration,
            &ws_threshold.ws_con_flicker_ban_reason,
            time,
            &mut self.banned_until,
        );

        if matches!(
            allow,
            AllowCon::Banned(_) | AllowCon::AlreadyBanned | AllowCon::Blocked
        ) {
            return allow;
        }

        let result = self.con_throttle.inc(
            &ws_threshold.ws_max_con_threshold,
            ws_threshold.ws_max_con_ban_reason,
            ws_threshold.ws_max_con_ban_duration,
            time,
            &mut self.banned_until,
        );

        if matches!(result, AllowCon::Allow) {
            self.con_flicker_throttle.inc();
        }

        result
    }
}

impl From<&WsThrottleCon> for TempThrottleConnection {
    fn from(value: &WsThrottleCon) -> Self {
        Self {
            banned_until: value.banned_until,
            con_flicker_throttle: value.con_flicker_throttle.clone(),
            con_throttle: value.con_throttle.clone(),
        }
    }
}

#[derive(Error, Debug)]
pub enum WsThrottleErr {
    #[error("MainGallery error: {0}")]
    Serialization(#[from] bincode::Error),
}

// #[derive(Error, Debug)]
// pub enum WsStatsOnMsgErr {
//     #[error("MainGallery error: {0}")]
//     Serialization(#[from] bincode::Error),

//     #[error("checl_throttle send error")]
//     SendCheckThrottle,

//     #[error("dsync sync send error")]
//     SendDiscSync,

//     #[error("Send error: {0}")]
//     SendToWsApp(#[from] tokio::sync::mpsc::error::SendError<WsAppMsg>),

//     #[error("Send error: {0}")]
//     Send(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),

//     #[error("Send error: {0}")]
//     ConnectionSend(#[from] tokio::sync::mpsc::error::SendError<ConMsg>),

//     #[error("Mongodb error: {0}")]
//     MongoDB(#[from] mongodb::error::Error),
// }
