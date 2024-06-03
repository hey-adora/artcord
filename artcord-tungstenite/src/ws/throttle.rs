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
use artcord_state::misc::throttle_threshold::is_banned;
use artcord_state::misc::throttle_threshold::AllowCon;
use artcord_state::misc::throttle_threshold::IsBanned;
use artcord_state::misc::throttle_threshold::Threshold;
use artcord_state::misc::throttle_threshold::ThrottleRanged;
use artcord_state::misc::throttle_threshold::ThrottleSimple;
use artcord_state::model::ws_statistics::ReqStat;
use artcord_state::model::ws_statistics::TempConIdType;
use artcord_state::util::time::time_is_past;
use artcord_state::util::time::time_passed_days;
use artcord_state::util::time::TimeMiddleware;
use artcord_state::ws::WsIpStat;
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

use super::con::throttle_stats_listener_tracker::ThrottleStatsListenerTracker;
use super::con::ConMsg;
use super::con::GlobalConMsg;
use super::con::IpConMsg;
use super::con::IpManagerMsg;
use super::WsAppMsg;

#[derive(Debug)]
pub struct WsThrottle {
    pub ips: HashMap<IpAddr, WsThrottleCon>,
    //pub stats_listeners: ThrottleStatsListenerTracker,
}

#[derive(Debug)]
pub struct WsThrottleCon {
    pub stats: WsIpStat,
    pub con_throttle: ThrottleRanged,
    pub con_flicker_throttle: ThrottleSimple,
    pub ip_con_tx: broadcast::Sender<IpConMsg>,
    pub ip_con_rx: broadcast::Receiver<IpConMsg>,
    pub ip_manager_tx: mpsc::Sender<IpManagerMsg>,
    pub ip_manager_task: JoinHandle<()>,
    //pub ip_req
    //pub stats_listeners: broadcast::Receiver<GlobalConMsg>,
}

#[derive(Debug)]
pub struct WsIpTask<TimeMiddlewareType: TimeMiddleware + Clone + Sync + Send + 'static> {
    stats: ReqStat,
    banned_until: Option<(DateTime<Utc>, IpBanReason)>,
    cancelation_token: CancellationToken,
    time_middleware: TimeMiddlewareType,
    ban_threshold: Threshold,
    ban_duration: TimeDelta,
    data_sync_rx: mpsc::Receiver<IpManagerMsg>,
    //pub ip_req
    //pub stats_listeners: broadcast::Receiver<GlobalConMsg>,
}

impl<TimeMiddlewareType: TimeMiddleware + Clone + Sync + Send + 'static>
    WsIpTask<TimeMiddlewareType>
{
    pub async fn manage_ip(
        cancelation_token: CancellationToken,
        data_sync_rx: mpsc::Receiver<IpManagerMsg>,
        time_middleware: TimeMiddlewareType,
        ban_threshold: Threshold,
        ban_duration: TimeDelta,
    ) {
        let mut task = Self {
            stats: ReqStat::new(),
            banned_until: None,
            cancelation_token,
            time_middleware,
            ban_duration,
            ban_threshold,
            data_sync_rx,
        };

        task.run().await;
    }

    pub async fn run(&mut self) {
        trace!("task is running");
        loop {
            select! {
                msg = self.data_sync_rx.recv() => {
                    let Some(msg) = msg else {
                        break;
                    };
                    let exit = self.on_msg(msg).await;
                    if exit {
                        break;
                    }
                }
                _ = self.cancelation_token.cancelled() => {
                    break;
                }
            }
        }
        trace!("task exited");
    }

    async fn on_msg(&mut self, msg: IpManagerMsg) -> bool {
        trace!("recv: {:#?}", &msg);
        match msg {
            IpManagerMsg::CheckThrottle {
                path,
                block_threshold,
                allow_tx,
            } => {
                let time = self.time_middleware.get_time().await;
                let allow = self.stats
                    .inc_path(
                        path,
                        &block_threshold,
                        &self.ban_threshold,
                        &self.ban_duration,
                        &mut self.banned_until,
                        &time,
                    )
                    .await;
                let send_result = allow_tx.send(allow);
                if send_result.is_err() {
                    error!("failed to send AllowCon");
                }
            }
            IpManagerMsg::Unban => {
                self.banned_until = None;
            }
        }
        trace!("recv finished");
        false
    }
}

impl WsThrottle {
    pub fn new() -> Self {
        Self {
            ips: HashMap::new(),
            //stats_listeners: ThrottleStatsListenerTracker::new(),
        }
    }
    pub fn ban(&mut self, ip: &IpAddr, ban_reason: IpBanReason, until: DateTime<Utc>) -> Result<(), tokio::sync::broadcast::error::SendError<IpConMsg>>{
        let ip_stats = self.ips.get_mut(ip);
        let Some(ip_stats) = ip_stats else {
            error!("throttle: cant be banned because it doesnt exist in the list");
            return Ok(());
        };
        ip_stats
            .con_throttle
            .ban(&mut ip_stats.stats.banned_until, ban_reason, until);
        ip_stats.ip_con_tx.send(IpConMsg::Disconnect)?;

        Ok(())
    }

    pub fn unban_on_throttle(&mut self, ip: &IpAddr) {
        let ip_stats = self.ips.get_mut(ip);
        let Some(ip_stats) = ip_stats else {
            error!("throttle: cant be banned because it doesnt exist in the list");
            return;
        };
        ip_stats
            .con_throttle
            .unban_on_throttle(&mut ip_stats.stats.banned_until);
    }

    pub async fn unban_on_ip_manager(&mut self, ip: &IpAddr) -> Result<(), tokio::sync::mpsc::error::SendError<IpManagerMsg>> {
        let ip_stats = self.ips.get_mut(ip);
        let Some(ip_stats) = ip_stats else {
            error!("throttle: cant be banned because it doesnt exist in the list");
            return Ok(());
        };
        ip_stats
        .ip_manager_tx.send(IpManagerMsg::Unban).await?;

        Ok(())
    }

    pub fn dec_con(&mut self, ip: &IpAddr, time: &DateTime<Utc>) {
        let ip_stats = self.ips.get_mut(ip);
        let Some(ip_stats) = ip_stats else {
            error!("throttle: cant disconnect ip that doesnt exist");
            return;
        };
        ip_stats.dec();
        if ip_stats.con_throttle.amount == 0 && ip_stats.is_banned(time) != IsBanned::Banned {
            self.ips.remove(&ip);
        }
        trace!("throttle on DEC: {:#?}", self);
    }

    pub fn get_total_allowed(&mut self, ip: &IpAddr) -> Option<u64> {
        let Some(con) = self.ips.get_mut(ip) else {
            return None;
        };
        Some(con.stats.total_allow_amount)
    }

    pub fn get_total_blocked(&mut self, ip: &IpAddr) -> Option<u64> {
        let Some(con) = self.ips.get_mut(ip) else {
            return None;
        };
        Some(con.stats.total_block_amount)
    }

    pub fn get_total_banned(&mut self, ip: &IpAddr) -> Option<u64> {
        let Some(con) = self.ips.get_mut(ip) else {
            return None;
        };
        Some(con.stats.total_banned_amount)
    }

    // pub fn get_total_unbanned(&mut self, ip: &IpAddr) -> Option<u64> {
    //     let Some(con) = self.ips.get_mut(ip) else {
    //         return None;
    //     };
    //     Some(con.stats.total_unbanned_amount)
    // }

    pub fn get_amounts(&mut self, ip: &IpAddr) -> Option<(u64, u64)> {
        let Some(con) = self.ips.get_mut(ip) else {
            return None;
        };
        Some((
            con.con_throttle.tracker.total_amount,
            con.con_throttle.tracker.amount,
        ))
    }

    pub fn get_ip_channel(
        &mut self,
        ip: &IpAddr,
    ) -> Option<(
        broadcast::Sender<IpConMsg>,
        broadcast::Receiver<IpConMsg>,
        mpsc::Sender<IpManagerMsg>,
    )> {
        let Some(con) = self.ips.get_mut(ip) else {
            return None;
        };

        Some((
            con.ip_con_tx.clone(),
            con.ip_con_rx.resubscribe(),
            con.ip_manager_tx.clone(),
        ))
    }

    pub fn inc_con<TimeMiddlewareType: TimeMiddleware + Clone + Sync + Send + 'static>(
        &mut self,
        ip: IpAddr,
        ws_threshold: &WsThreshold,
        task_tracker: &TaskTracker,
        cancellation_token: &CancellationToken,
        time: &DateTime<Utc>,
        time_middleware: &TimeMiddlewareType,
        // ban_threshold: &Threshold,
        // ban_duration: &TimeDelta,
    ) -> AllowCon {
        let con = self.ips.entry(ip).or_insert_with(|| {
            WsThrottleCon::new(
                ip,
                ws_threshold.ws_max_con_threshold_range,
                task_tracker,
                cancellation_token.clone(),
                *time,
                time_middleware.clone(),
                ws_threshold.ws_req_ban_threshold.clone(),
                ws_threshold.ws_req_ban_duration.clone(),
            )
        });

        let result = con.inc(ws_threshold, time);
        match result {
            AllowCon::Allow => {
                con.stats.total_allow_amount += 1;
            }
            AllowCon::Blocked | AllowCon::UnbannedAndBlocked => {
                con.stats.total_block_amount += 1;
            }
            // AllowCon::Blocked => {
            //     con.stats.total_block_amount += 1;
            // }
            AllowCon::Banned(_) => {
                con.stats.total_banned_amount += 1;
            }
            AllowCon::AlreadyBanned => {
                con.stats.total_already_banned_amount += 1;
            }

            AllowCon::UnbannedAndAllow => {
                //con.stats.total_unbanned_amount += 1;
            }
        }
        trace!("throttle result {:?} and INC: {:#?}", result, self);
        result
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

    pub fn new<TimeMiddlewareType: TimeMiddleware + Clone + Sync + Send + 'static>(
        ip: IpAddr,
        range: u64,
        task_tracker: &TaskTracker,
        cancelation_token: CancellationToken,
        started_at: DateTime<Utc>,
        time_middleware: TimeMiddlewareType,
        ban_threshold: Threshold,
        ban_duration: TimeDelta,
    ) -> Self {
        
        let (con_broadcast_tx, con_broadcast_rx) = broadcast::channel(1);
        let (ip_data_sync_tx, ip_data_sync_rx) = mpsc::channel(1);
        let ip_data_sync_task = task_tracker.spawn(
            WsIpTask::manage_ip(
                cancelation_token,
                ip_data_sync_rx,
                time_middleware,
                ban_threshold,
                ban_duration,
            )
            .instrument(tracing::trace_span!("ip_sync", "{}", ip)),
        );
        let con = Self {
            //path_stats: HashMap::new(),
            stats: WsIpStat::new(ip),
            con_throttle: ThrottleRanged::new(range, started_at),
            con_flicker_throttle: ThrottleSimple::new(started_at),

            ip_con_tx: con_broadcast_tx,
            ip_con_rx: con_broadcast_rx,
            ip_manager_tx: ip_data_sync_tx,
            ip_manager_task: ip_data_sync_task,
            // ip_stats_tx: ip_stats_tx.clone(),
            // ip_stats_rx: ip_stats_rx.clone(),
        };
        // ((ip_stats_tx, ip_stats_rx), con)
        con
    }

    pub fn is_banned(&mut self, time: &DateTime<Utc>) -> IsBanned {
        is_banned(&mut self.stats.banned_until, time)
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
            &mut self.stats.banned_until,
        );

        trace!("throttle: flicker throttle result: {:?}", allow);

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
            &mut self.stats.banned_until,
        );

        trace!("throttle: result: {:?}", result);

        match result {
            AllowCon::Allow => {
                self.con_flicker_throttle.inc();
                if allow == AllowCon::UnbannedAndAllow {
                    allow
                } else {
                    result
                }
            }
            AllowCon::Blocked => {
                if allow == AllowCon::UnbannedAndAllow {
                    AllowCon::UnbannedAndBlocked
                } else {
                    result
                }
            }
            // AllowCon::Blocked => {
            //     if allow == AllowCon::Unbanned {
            //         AllowCon::UnbannedAndBlocked
            //     } else {
            //         result
            //     }
            // }
            _ => result,
        }
    }
}

impl From<&WsThrottleCon> for TempThrottleConnection {
    fn from(value: &WsThrottleCon) -> Self {
        Self {
            banned_until: value.stats.banned_until,
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

#[cfg(test)]
mod throttle_tests {
    use artcord_state::{misc::{
        throttle_connection::IpBanReason,
        throttle_threshold::{AllowCon, Threshold},
    }, util::time::Clock};
    use chrono::{TimeDelta, Utc};
    use std::net::{IpAddr, Ipv4Addr};
    use std::str::FromStr;
    use tokio_util::{sync::CancellationToken, task::TaskTracker};
    use tracing::{debug, trace};

    use crate::WsThreshold;

    use super::WsThrottle;

    #[tokio::test]
    async fn ws_throttle_test() {
        let _ = tracing_subscriber::fmt()
            .event_format(
                tracing_subscriber::fmt::format()
                    .with_file(true)
                    .with_line_number(true),
            )
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
                    .unwrap_or(tracing_subscriber::EnvFilter::from_str("artcord=trace").unwrap()),
            )
            .try_init();

        let mut throttle = WsThrottle::new();
        let mut time = Utc::now();
        let ws_threshold = WsThreshold {
            ws_max_con_threshold: Threshold::new_const(10, TimeDelta::try_minutes(1)),
            ws_max_con_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
            ws_max_con_threshold_range: 5,
            ws_max_con_ban_reason: IpBanReason::WsTooManyReconnections,
            ws_con_flicker_threshold: Threshold::new_const(20, TimeDelta::try_minutes(1)),
            ws_con_flicker_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
            ws_con_flicker_ban_reason: IpBanReason::WsConFlickerDetected,
            ws_req_ban_threshold: Threshold::new_const(1, TimeDelta::try_minutes(1)),
            ws_req_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
        };
        let ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 69));
        let task_tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let time_middleware = Clock::new();

        for _ in 0..5 {
            let con_1 =
                throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
            time += TimeDelta::try_minutes(1).unwrap();
            assert_eq!(con_1, AllowCon::Allow);
        }
        //time += TimeDelta::try_minutes(10).unwrap();
        let con_1 = throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
        assert_eq!(con_1, AllowCon::Blocked);

        //time += TimeDelta::try_minutes(2).unwrap();

        throttle.dec_con(&ip, &time);

        let con_1 = throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
        assert_eq!(con_1, AllowCon::Allow);

        for _ in 0..19 {
            throttle.dec_con(&ip, &time);
            let con_1 =
                throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
            assert_eq!(con_1, AllowCon::Allow);
        }

        throttle.dec_con(&ip, &time);

        let con_1 = throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
        assert_eq!(
            con_1,
            AllowCon::Banned((
                time + TimeDelta::try_minutes(1).unwrap(),
                IpBanReason::WsConFlickerDetected
            ))
        );

        let con_1 = throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
        assert_eq!(con_1, AllowCon::AlreadyBanned);

        time += TimeDelta::try_minutes(1).unwrap();

        let con_1 = throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
        assert_eq!(con_1, AllowCon::UnbannedAndAllow);

        for _ in 0..10 {
            let con_1 =
                throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
            assert_eq!(con_1, AllowCon::Blocked);
        }

        let con_1 = throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
        assert_eq!(
            con_1,
            AllowCon::Banned((
                time + TimeDelta::try_minutes(1).unwrap(),
                IpBanReason::WsTooManyReconnections
            ))
        );

        let con_1 = throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
        assert_eq!(con_1, AllowCon::AlreadyBanned);

        time += TimeDelta::try_minutes(1).unwrap();

        debug!("ONE: {:#?}", throttle);
        let con_1 = throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
        assert_eq!(con_1, AllowCon::UnbannedAndBlocked);
        debug!("TWO: {:#?}", throttle);

        throttle.dec_con(&ip, &time);
        debug!("THREE: {:#?}", throttle);

        let con_1 = throttle.inc_con(ip, &ws_threshold, &task_tracker, &cancellation_token, &time, &time_middleware);
        assert_eq!(con_1, AllowCon::Allow);

        for _ in 0..5 {
            throttle.dec_con(&ip, &time);
        }

        let ip_exists = throttle.ips.get(&ip).is_some();
        assert!(!ip_exists);

        //trace!("throttle: {:#?}", throttle);
    }
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
