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
use artcord_state::global;
use artcord_state::global::Threshold;
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
use super::WsIp;

#[derive(Debug, Clone, PartialEq)]
pub enum AllowCon {
    Allow,
    Blocked,
    AlreadyBanned,
    Banned((DateTime<Utc>, global::IpBanReason)),
    UnbannedAndAllow,
    UnbannedAndBlocked,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IsBanned {
    Banned,
    NotBanned,
    UnBanned,
}

// impl<TimeMiddlewareType: global::TimeMiddleware + Clone + Sync + Send + 'static>
//     WsIpTask<TimeMiddlewareType>
// {
//     pub async fn manage_ip(
//         cancelation_token: CancellationToken,
//         data_sync_rx: mpsc::Receiver<IpManagerMsg>,
//         time_middleware: TimeMiddlewareType,
//         ban_threshold: global::Threshold,
//         ban_duration: TimeDelta,
//     ) {
//         let mut task = Self {
//             stats: WsConReqStats::new(),
//             banned_until: None,
//             cancelation_token,
//             time_middleware,
//             ban_duration,
//             ban_threshold,
//             data_sync_rx,
//         };

//         task.run().await;
//     }

//     pub async fn run(&mut self) {
//         trace!("task is running");
//         loop {
//             select! {
//                 msg = self.data_sync_rx.recv() => {
//                     let Some(msg) = msg else {
//                         break;
//                     };
//                     let exit = self.on_msg(msg).await;
//                     if exit {
//                         break;
//                     }
//                 }
//                 _ = self.cancelation_token.cancelled() => {
//                     break;
//                 }
//             }
//         }
//         trace!("task exited");
//     }

//     async fn on_msg(&mut self, msg: IpManagerMsg) -> bool {
//         trace!("recv: {:#?}", &msg);
//         match msg {
//             IpManagerMsg::CheckThrottle {
//                 path,
//                 block_threshold,
//                 allow_tx,
//             } => {
//                 let time = self.time_middleware.get_time().await;
//                 let allow = path_throttle_check(
//                     &mut self.stats,
//                     path,
//                     &block_threshold,
//                     &self.ban_threshold,
//                     &self.ban_duration,
//                     &mut self.banned_until,
//                     &time,
//                 )
//                 .await;
//                 let send_result = allow_tx.send(allow);
//                 if send_result.is_err() {
//                     error!("failed to send AllowCon");
//                 }
//             }
//             IpManagerMsg::Unban => {
//                 self.banned_until = None;
//             }
//         }
//         trace!("recv finished");
//         false
//     }
// }

// impl WsIpTracker {
//     pub fn new() -> Self {
//         Self {
//             ips: HashMap::new(),
//             //stats_listeners: ThrottleStatsListenerTracker::new(),
//         }
//     }
//     pub fn ban(
//         &mut self,
//         ip: &IpAddr,
//         ban_reason: global::IpBanReason,
//         until: DateTime<Utc>,
//     ) -> Result<(), tokio::sync::broadcast::error::SendError<IpConMsg>> {
//         let ip_stats = self.ips.get_mut(ip);
//         let Some(ip_stats) = ip_stats else {
//             error!("throttle: cant be banned because it doesnt exist in the list");
//             return Ok(());
//         };
//         ip_stats
//             .con_throttle
//             .ban(&mut ip_stats.stats.banned_until, ban_reason, until);
//         ip_stats.ip_con_tx.send(IpConMsg::Disconnect)?;

//         Ok(())
//     }

//     pub fn unban_on_throttle(&mut self, ip: &IpAddr) {
//         let ip_stats = self.ips.get_mut(ip);
//         let Some(ip_stats) = ip_stats else {
//             error!("throttle: cant be banned because it doesnt exist in the list");
//             return;
//         };
//         ip_stats
//             .con_throttle
//             .unban_on_throttle(&mut ip_stats.stats.banned_until);
//     }

//     pub async fn unban_on_ip_manager(
//         &mut self,
//         ip: &IpAddr,
//     ) -> Result<(), tokio::sync::mpsc::error::SendError<IpManagerMsg>> {
//         let ip_stats = self.ips.get_mut(ip);
//         let Some(ip_stats) = ip_stats else {
//             error!("throttle: cant be banned because it doesnt exist in the list");
//             return Ok(());
//         };
//         ip_stats.ip_manager_tx.send(IpManagerMsg::Unban).await?;

//         Ok(())
//     }

//     pub fn dec_con(&mut self, ip: &IpAddr, time: &DateTime<Utc>) {
//         let ip_stats = self.ips.get_mut(ip);
//         let Some(ip_stats) = ip_stats else {
//             error!("throttle: cant disconnect ip that doesnt exist");
//             return;
//         };
//         ip_stats.dec();
//         if ip_stats.con_throttle.amount == 0 && ip_stats.is_banned(time) != IsBanned::Banned {
//             self.ips.remove(&ip);
//         }
//         trace!("throttle on DEC: {:#?}", self);
//     }

//     pub fn get_total_allowed(&mut self, ip: &IpAddr) -> Option<u64> {
//         let Some(con) = self.ips.get_mut(ip) else {
//             return None;
//         };
//         Some(con.stats.total_allow_amount)
//     }

//     pub fn get_total_blocked(&mut self, ip: &IpAddr) -> Option<u64> {
//         let Some(con) = self.ips.get_mut(ip) else {
//             return None;
//         };
//         Some(con.stats.total_block_amount)
//     }

//     pub fn get_total_banned(&mut self, ip: &IpAddr) -> Option<u64> {
//         let Some(con) = self.ips.get_mut(ip) else {
//             return None;
//         };
//         Some(con.stats.total_banned_amount)
//     }

//     // pub fn get_total_unbanned(&mut self, ip: &IpAddr) -> Option<u64> {
//     //     let Some(con) = self.ips.get_mut(ip) else {
//     //         return None;
//     //     };
//     //     Some(con.stats.total_unbanned_amount)
//     // }

//     pub fn get_amounts(&mut self, ip: &IpAddr) -> Option<(u64, u64)> {
//         let Some(con) = self.ips.get_mut(ip) else {
//             return None;
//         };
//         Some((
//             con.con_throttle.tracker.total_amount,
//             con.con_throttle.tracker.amount,
//         ))
//     }

//     pub fn get_ip_channel(
//         &mut self,
//         ip: &IpAddr,
//     ) -> Option<(
//         broadcast::Sender<IpConMsg>,
//         broadcast::Receiver<IpConMsg>,
//         mpsc::Sender<IpManagerMsg>,
//     )> {
//         let Some(con) = self.ips.get_mut(ip) else {
//             return None;
//         };

//         Some((
//             con.ip_con_tx.clone(),
//             con.ip_con_rx.resubscribe(),
//             con.ip_manager_tx.clone(),
//         ))
//     }

// }

// pub fn con_connect_throttle_check<TimeMiddlewareType: global::TimeMiddleware + Clone + Sync + Send + 'static>(
//     ips: &mut HashMap<IpAddr, WsIp>,
//     ip: IpAddr,
//     ws_threshold: &WsThreshold,
//     task_tracker: &TaskTracker,
//     cancellation_token: &CancellationToken,
//     time: &DateTime<Utc>,
//     time_middleware: &TimeMiddlewareType,
//     // ban_threshold: &Threshold,
//     // ban_duration: &TimeDelta,
// ) -> AllowCon {

//     let result = con.inc(ws_threshold, time);
//     match result {
//         AllowCon::Allow => {
//             con.stats.total_allow_amount += 1;
//         }
//         AllowCon::Blocked | AllowCon::UnbannedAndBlocked => {
//             con.stats.total_block_amount += 1;
//         }
//         // AllowCon::Blocked => {
//         //     con.stats.total_block_amount += 1;
//         // }
//         AllowCon::Banned(_) => {
//             con.stats.total_banned_amount += 1;
//         }
//         AllowCon::AlreadyBanned => {
//             con.stats.total_already_banned_amount += 1;
//         }

//         AllowCon::UnbannedAndAllow => {
//             //con.stats.total_unbanned_amount += 1;
//         }
//     }
//     trace!("throttle result {:?} and INC: {:#?}", result, ips);
//     result
// }

// pub fn con_disconnect_throttle_check(
//     ips: &mut HashMap<IpAddr, WsIp>,
//     ip: &IpAddr,
//     time: &DateTime<Utc>,
// ) {
//     let ip_stats = ips.get_mut(ip);
//     let Some(ip_stats) = ip_stats else {
//         error!("throttle: cant disconnect ip that doesnt exist");
//         return;
//     };
//     ip_stats.dec();
//     if ip_stats.con_throttle.amount == 0 && ip_stats.is_banned(time) != IsBanned::Banned {
//         ips.remove(&ip);
//     }
//     trace!("throttle on DEC: {:#?}", ips);
// }

// impl WsThrottleCon {
//     // pub fn to_temp(
//     //     value: &HashMap<IpAddr, WsThrottleCon>,
//     // ) -> HashMap<IpAddr, TempThrottleConnection> {
//     //     value
//     //         .into_iter()
//     //         .fold(HashMap::new(), |mut a, (key, value)| {
//     //             a.insert(*key, value.into());
//     //             a
//     //         })
//     // }

//     pub fn new<TimeMiddlewareType: global::TimeMiddleware + Clone + Sync + Send + 'static>(
//         ip: IpAddr,
//         range: u64,
//         task_tracker: &TaskTracker,
//         cancelation_token: CancellationToken,
//         started_at: DateTime<Utc>,
//         time_middleware: TimeMiddlewareType,
//         ban_threshold: global::Threshold,
//         ban_duration: TimeDelta,
//     ) -> Self {
//         let (con_broadcast_tx, con_broadcast_rx) = broadcast::channel(1);
//         let (ip_data_sync_tx, ip_data_sync_rx) = mpsc::channel(1);
//         let ip_data_sync_task = task_tracker.spawn(
//             WsIpTask::manage_ip(
//                 cancelation_token,
//                 ip_data_sync_rx,
//                 time_middleware,
//                 ban_threshold,
//                 ban_duration,
//             )
//             .instrument(tracing::trace_span!("ip_sync", "{}", ip)),
//         );
//         let con = Self {
//             //path_stats: HashMap::new(),
//             stats: global::WsIpStat::new(ip),
//             con_throttle: global::ThrottleRanged::new(range, started_at),
//             con_flicker_throttle: global::ThrottleSimple::new(started_at),

//             ip_con_tx: con_broadcast_tx,
//             ip_con_rx: con_broadcast_rx,
//             ip_manager_tx: ip_data_sync_tx,
//             ip_manager_task: ip_data_sync_task,
//             // ip_stats_tx: ip_stats_tx.clone(),
//             // ip_stats_rx: ip_stats_rx.clone(),
//         };
//         // ((ip_stats_tx, ip_stats_rx), con)
//         con
//     }

//     pub fn dec(&mut self) {
//         self.con_throttle.dec();
//     }

//     pub fn inc(&mut self, ws_threshold: &WsThreshold, time: &DateTime<Utc>) -> AllowCon {
//         let allow = self.con_flicker_throttle.allow(
//             &ws_threshold.ws_con_flicker_threshold,
//             &ws_threshold.ws_con_flicker_ban_duration,
//             &ws_threshold.ws_con_flicker_ban_reason,
//             time,
//             &mut self.stats.banned_until,
//         );

//         trace!("throttle: flicker throttle result: {:?}", allow);

//         if matches!(
//             allow,
//             AllowCon::Banned(_) | AllowCon::AlreadyBanned | AllowCon::Blocked
//         ) {
//             return allow;
//         }

//         let result = self.con_throttle.inc(
//             &ws_threshold.ws_max_con_threshold,
//             ws_threshold.ws_max_con_ban_reason,
//             ws_threshold.ws_max_con_ban_duration,
//             time,
//             &mut self.stats.banned_until,
//         );

//         trace!("throttle: result: {:?}", result);

//         match result {
//             AllowCon::Allow => {
//                 self.con_flicker_throttle.inc();
//                 if allow == AllowCon::UnbannedAndAllow {
//                     allow
//                 } else {
//                     result
//                 }
//             }
//             AllowCon::Blocked => {
//                 if allow == AllowCon::UnbannedAndAllow {
//                     AllowCon::UnbannedAndBlocked
//                 } else {
//                     result
//                 }
//             }
//             // AllowCon::Blocked => {
//             //     if allow == AllowCon::Unbanned {
//             //         AllowCon::UnbannedAndBlocked
//             //     } else {
//             //         result
//             //     }
//             // }
//             _ => result,
//         }
//     }
// }

pub fn ws_ip_throttle(
    con_flicker_tracker: &mut global::ThresholdTracker,
    con_count_tracker: &mut global::ThresholdTracker,
    current_con_count: &mut u64,
    banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
    ws_threshold: &WsThreshold,
    time: &DateTime<Utc>,
) -> AllowCon {
    //let flicker_throttle_allow = threshold_allow(tracker, flicker_threshold, time);
    let flicker_throttle_allow = simple_throttle(
        con_flicker_tracker,
        &ws_threshold.ws_max_con_threshold,
        &ws_threshold.ws_max_con_ban_duration,
        &ws_threshold.ws_max_con_ban_reason,
        time,
        banned_until,
    );

    // let allow = con_flicker_throttle.allow(
    //     &ws_threshold.ws_con_flicker_threshold,
    //     &ws_threshold.ws_con_flicker_ban_duration,
    //     &ws_threshold.ws_con_flicker_ban_reason,
    //     time,
    //     banned_until,
    // );

    trace!(
        "throttle: flicker throttle result: {:?}",
        flicker_throttle_allow
    );

    if matches!(
        flicker_throttle_allow,
        AllowCon::Banned(_) | AllowCon::AlreadyBanned | AllowCon::Blocked
    ) {
        return flicker_throttle_allow;
    }

    let ranged_throttle_allow = ranged_throttle(
        &ws_threshold.ws_max_con_threshold_range,
        current_con_count,
        con_count_tracker,
        &ws_threshold.ws_con_flicker_threshold,
        &ws_threshold.ws_max_con_ban_reason,
        &ws_threshold.ws_max_con_ban_duration,
        time,
        banned_until,
    );
    // let result = con_throttle.inc(
    //     &ws_threshold.ws_max_con_threshold,
    //     ws_threshold.ws_max_con_ban_reason,
    //     ws_threshold.ws_max_con_ban_duration,
    //     time,
    //     banned_until,
    // );

    trace!("throttle: result: {:?}", ranged_throttle_allow);

    match ranged_throttle_allow {
        AllowCon::Allow => {
            con_flicker_tracker.amount += 1;
            if flicker_throttle_allow == AllowCon::UnbannedAndAllow {
                flicker_throttle_allow
            } else {
                ranged_throttle_allow
            }
        }
        AllowCon::Blocked => {
            if flicker_throttle_allow == AllowCon::UnbannedAndAllow {
                AllowCon::UnbannedAndBlocked
            } else {
                ranged_throttle_allow
            }
        }
        // AllowCon::Blocked => {
        //     if allow == AllowCon::Unbanned {
        //         AllowCon::UnbannedAndBlocked
        //     } else {
        //         result
        //     }
        // }
        _ => ranged_throttle_allow,
    }
}

// impl From<&WsThrottleCon> for TempThrottleConnection {
//     fn from(value: &WsThrottleCon) -> Self {
//         Self {
//             banned_until: value.stats.banned_until,
//             con_flicker_throttle: value.con_flicker_throttle.clone(),
//             con_throttle: value.con_throttle.clone(),
//         }
//     }
// }

// pub async fn req_throttle(
//     req_stat: &mut global::WsConReqStat,
//     path: global::ClientPathType,
//     block_threshold: &global::Threshold,
//     ban_threshold: &global::Threshold,
//     ban_duration: &TimeDelta,
//     banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
//     time: &DateTime<Utc>,
// ) -> AllowCon {
//     let path = req_stat
//         .req_stats
//         .entry(path)
//         .or_insert_with(|| global::WsConReqStat::new(*time));

//     let result = double_tracker_check(
//         &path.throttle.block_tracker,
//         &path.throttle.ban_tracker,
//         block_threshold,
//         ban_threshold,
//         global::IpBanReason::WsRouteBruteForceDetected,
//         ban_duration,
//         time,
//         banned_until,
//     );

//     //path.total_count += 1;

//     match &result {
//         AllowCon::Allow | AllowCon::UnbannedAndAllow => {
//             path.total_allowed_count += 1;
//         }
//         AllowCon::Blocked | AllowCon::UnbannedAndBlocked => {
//             path.total_blocked_count += 1;
//         }
//         AllowCon::Banned(_) => {
//             path.total_banned_count += 1;
//         }
//         AllowCon::AlreadyBanned => {
//             path.total_already_banned_count += 1;
//         }
//     }

//     result
// }

pub fn double_throttle(
    block_tracker: &mut global::ThresholdTracker,
    ban_tracker: &mut global::ThresholdTracker,
    block_threshold: &global::Threshold,
    ban_threshold: &global::Threshold,
    ban_reason: global::IpBanReason,
    ban_duration: &TimeDelta,
    time: &DateTime<Utc>,
    banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
) -> AllowCon {
    let ban_status = is_banned(banned_until, time);
    match ban_status {
        IsBanned::Banned => {
            return AllowCon::AlreadyBanned;
        }
        IsBanned::UnBanned => {
            ban_tracker.started_at = *time;
            ban_tracker.amount = 0;

            block_tracker.started_at = *time;
            block_tracker.amount = 0;
        }
        IsBanned::NotBanned => {}
    }

    if !threshold_allow(ban_tracker, ban_threshold, time) {
        let ban_until = *time + *ban_duration;
        *banned_until = Some((ban_until, ban_reason));
        return AllowCon::Banned((ban_until, ban_reason));
    }

    if !threshold_allow(block_tracker, block_threshold, time) {
        ban_tracker.amount += 1;
        return if ban_status == IsBanned::UnBanned {
            AllowCon::UnbannedAndBlocked
        } else {
            AllowCon::Blocked
        };
    } else {
        block_tracker.amount += 1;
    }

    if ban_status == IsBanned::UnBanned {
        AllowCon::UnbannedAndAllow
    } else {
        AllowCon::Allow
    }
}

pub fn ranged_throttle(
    max: &u64,
    current: &mut u64,
    tracker: &mut global::ThresholdTracker,
    threshold: &global::Threshold,
    ban_reason: &global::IpBanReason,
    ban_duration: &TimeDelta,
    time: &DateTime<Utc>,
    banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
) -> AllowCon {
    let ban_status = is_banned(banned_until, time);
    trace!("throttle: ban status: {:?}", ban_status);

    match ban_status {
        IsBanned::Banned => {
            return AllowCon::AlreadyBanned;
        }
        IsBanned::UnBanned => {
            tracker.started_at = *time;
            tracker.amount = 0;
        }
        IsBanned::NotBanned => {}
    }

    trace!(
        "throttle: range {} >= {} = {}",
        current,
        max,
        *current >= *max
    );
    if *current >= *max {
        let range_status = threshold_allow(tracker, threshold, time);
        //let range_status = !self.tracker.allow(threshold, time);
        trace!("throttle: range allow: {}", range_status);

        if range_status {
            let ban_until = *time + *ban_duration;
            *banned_until = Some((ban_until, *ban_reason));
            return AllowCon::Banned((ban_until, *ban_reason));
        }

        tracker.amount += 1;

        return if ban_status == IsBanned::UnBanned {
            AllowCon::UnbannedAndBlocked
        } else {
            AllowCon::Blocked
        };
    }

    *current += 1;

    if ban_status == IsBanned::UnBanned {
        AllowCon::UnbannedAndAllow
    } else {
        AllowCon::Allow
    }
}

pub fn simple_throttle(
    tracker: &mut global::ThresholdTracker,
    threshold: &global::Threshold,
    ban_duration: &TimeDelta,
    ban_reason: &global::IpBanReason,
    time: &DateTime<Utc>,
    banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
) -> AllowCon {
    match is_banned(banned_until, time) {
        IsBanned::Banned => {
            return AllowCon::AlreadyBanned;
        }
        IsBanned::UnBanned => {
            tracker.started_at = *time;
            tracker.amount = 0;
            return AllowCon::UnbannedAndAllow;
        }
        _ => {}
    }
    let allow = threshold_allow(tracker, threshold, time);
    if !allow {
        let ban = (*time + *ban_duration, *ban_reason);
        *banned_until = Some(ban);
        return AllowCon::Banned(ban);
    }

    AllowCon::Allow
}

pub fn threshold_allow(
    tracker: &mut global::ThresholdTracker,
    threshold: &global::Threshold,
    time: &DateTime<Utc>,
) -> bool {
    let max_reatched = tracker.amount >= threshold.amount;
    let time_passed = (*time - tracker.started_at) >= threshold.delta;
    trace!("threshold_allow: max_reatched: {}, time_passed: {}", max_reatched, time_passed);

    if time_passed {
        tracker.started_at = *time;
        tracker.amount = 0;
    }
    !max_reatched || time_passed
}

pub fn compare_pick_worst(a: AllowCon, b: AllowCon) -> AllowCon {
    let get_order = |v: &AllowCon| match v {
        AllowCon::AlreadyBanned => 5,
        AllowCon::Banned(_) => 4,
        AllowCon::UnbannedAndBlocked => 3,
        AllowCon::Blocked => 2,
        AllowCon::UnbannedAndAllow => 1,
        AllowCon::Allow => 0,
    };
    let a_level = get_order(&a);
    let b_level = get_order(&b);
    if a_level >= b_level {
        a
    } else {
        b
    }
}

pub fn is_banned(
    banned_until: &mut Option<(DateTime<Utc>, global::IpBanReason)>,
    time: &DateTime<Utc>,
) -> IsBanned {
    let Some((date, _)) = banned_until else {
        trace!("throttle: ban check: entry doesnt exist");
        return IsBanned::NotBanned;
    };

    let un_banned = time >= date;

    trace!(
        "throttle: is banned: {}, state: {:#?}",
        !un_banned,
        banned_until
    );

    if un_banned {
        *banned_until = None;
        return IsBanned::UnBanned;
    }
    IsBanned::Banned
}

#[derive(Error, Debug)]
pub enum WsThrottleErr {
    #[error("MainGallery error: {0}")]
    Serialization(#[from] bincode::Error),
}

#[cfg(test)]
mod throttle_tests {
    use artcord_state::global;
    use chrono::{DateTime, TimeDelta, Utc};
    use std::net::{IpAddr, Ipv4Addr};
    use std::str::FromStr;
    use tokio_util::{sync::CancellationToken, task::TaskTracker};
    use tracing::{debug, trace};

    use crate::ws::throttle::{double_throttle, ranged_throttle, ws_ip_throttle};
    use crate::WsThreshold;

    use super::{threshold_allow, AllowCon};

    #[tokio::test]
    async fn ws_throttle_test() {
        init_logger();

        //let mut throttle = WsIpTracker::new();
        let mut time = Utc::now();
        let ws_threshold = WsThreshold {
            ws_max_con_threshold: global::Threshold::new_const(10, TimeDelta::try_minutes(1)),
            ws_max_con_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
            ws_max_con_threshold_range: 5,
            ws_max_con_ban_reason: global::IpBanReason::WsTooManyReconnections,
            ws_con_flicker_threshold: global::Threshold::new_const(20, TimeDelta::try_minutes(1)),
            ws_con_flicker_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
            ws_con_flicker_ban_reason: global::IpBanReason::WsConFlickerDetected,
            ws_req_ban_threshold: global::Threshold::new_const(1, TimeDelta::try_minutes(1)),
            ws_req_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
        };

        let ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 69));
        let max_con_count: u64 = 5;
        let mut current_con_count: u64 = 0;
        let mut flicker_tracker = global::ThresholdTracker::new(time);
        let mut con_tracker = global::ThresholdTracker::new(time);
        let mut banned_until: Option<(DateTime<Utc>, global::IpBanReason)> = None;
        //let ranged_throttle = global::ThrottleRanged::new(range, started_at)
        // let task_tracker = TaskTracker::new();
        // let cancellation_token = CancellationToken::new();
        // let time_middleware = global::Clock::new();

        for _ in 0..5 {
            let con_1 = ws_ip_throttle(
                &mut flicker_tracker,
                &mut con_tracker,
                &mut current_con_count,
                &mut banned_until,
                &ws_threshold,
                &time,
            );
            time += TimeDelta::try_minutes(1).unwrap();
            assert_eq!(con_1, AllowCon::Allow);
        }
        //time += TimeDelta::try_minutes(10).unwrap();
        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::Blocked);

        //time += TimeDelta::try_minutes(2).unwrap();

        current_con_count -= 1;
        //throttle.dec_con(&ip, &time);

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::Allow);

        for _ in 0..19 {
            current_con_count -= 1;
            let con_1 = ws_ip_throttle(
                &mut flicker_tracker,
                &mut con_tracker,
                &mut current_con_count,
                &mut banned_until,
                &ws_threshold,
                &time,
            );
            assert_eq!(con_1, AllowCon::Allow);
        }

        current_con_count -= 1;

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(
            con_1,
            AllowCon::Banned((
                time + TimeDelta::try_minutes(1).unwrap(),
                global::IpBanReason::WsConFlickerDetected
            ))
        );

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::AlreadyBanned);

        time += TimeDelta::try_minutes(1).unwrap();

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::UnbannedAndAllow);

        for _ in 0..10 {
            let con_1 = ws_ip_throttle(
                &mut flicker_tracker,
                &mut con_tracker,
                &mut current_con_count,
                &mut banned_until,
                &ws_threshold,
                &time,
            );
            assert_eq!(con_1, AllowCon::Blocked);
        }

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(
            con_1,
            AllowCon::Banned((
                time + TimeDelta::try_minutes(1).unwrap(),
                global::IpBanReason::WsTooManyReconnections
            ))
        );

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::AlreadyBanned);

        time += TimeDelta::try_minutes(1).unwrap();

        //debug!("ONE: {:#?}", throttle);
        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::UnbannedAndBlocked);
        //debug!("TWO: {:#?}", throttle);

        current_con_count -= 1;
        //debug!("THREE: {:#?}", throttle);

        let con_1 = ws_ip_throttle(
            &mut flicker_tracker,
            &mut con_tracker,
            &mut current_con_count,
            &mut banned_until,
            &ws_threshold,
            &time,
        );
        assert_eq!(con_1, AllowCon::Allow);

        for _ in 0..5 {
            current_con_count -= 1;
        }

        // let ip_exists = throttle.ips.get(&ip).is_some();
        // assert!(!ip_exists);

        //trace!("throttle: {:#?}", throttle);
    }

    #[test]
    fn throttle_ranged_test() {
        init_logger();

        let time = Utc::now();
        let now = Utc::now();
        let ban_reason = global::IpBanReason::WsTooManyReconnections;
        let ban_duration = TimeDelta::try_seconds(10).unwrap();
        let mut banned_until: Option<(DateTime<Utc>, global::IpBanReason)> = None;

        let max = 10;
        let mut current = 0;
        let mut tracker = global::ThresholdTracker::new(time);
        let threshold = global::Threshold::new(10, TimeDelta::try_seconds(10).unwrap());
        let mut banned_until: Option<(DateTime<Utc>, global::IpBanReason)> = None;
        //let mut throttle = global::ThrottleRanged::new(10, started_at);

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!((result, current, tracker.amount,), (AllowCon::Allow, 1, 0));

        for _ in 0..8 {
            let result = ranged_throttle(
                &max,
                &mut current,
                &mut tracker,
                &threshold,
                &ban_reason,
                &ban_duration,
                &time,
                &mut banned_until,
            );
        }

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!((result, current, tracker.amount,), (AllowCon::Allow, 10, 0));

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (AllowCon::Blocked, 10, 1)
        );

        for _ in 0..9 {
            let result = ranged_throttle(
                &max,
                &mut current,
                &mut tracker,
                &threshold,
                &ban_reason,
                &ban_duration,
                &time,
                &mut banned_until,
            );
        }

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (
                AllowCon::Banned((
                    now.checked_add_signed(ban_duration).unwrap(),
                    global::IpBanReason::WsTooManyReconnections
                )),
                10,
                10
            )
        );

        let now = now.checked_add_signed(ban_duration).unwrap();
        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (AllowCon::UnbannedAndBlocked, 10, 1,)
        );

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (AllowCon::Blocked, 10, 2)
        );

        tracker.amount -= 1;

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!((result, current, tracker.amount,), (AllowCon::Allow, 10, 2));

        for _ in 0..8 {
            let result = ranged_throttle(
                &max,
                &mut current,
                &mut tracker,
                &threshold,
                &ban_reason,
                &ban_duration,
                &time,
                &mut banned_until,
            );
        }

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (
                AllowCon::Banned((
                    now.checked_add_signed(ban_duration).unwrap(),
                    global::IpBanReason::WsTooManyReconnections
                )),
                10,
                10
            )
        );

        let now = now.checked_add_signed(ban_duration).unwrap();
        tracker.amount -= 1;

        let result = ranged_throttle(
            &max,
            &mut current,
            &mut tracker,
            &threshold,
            &ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (result, current, tracker.amount,),
            (AllowCon::UnbannedAndAllow, 10, 0)
        );
    }

    #[test]
    fn throttle_double_layer_test() {
        init_logger();

        let time = Utc::now();
        let ban_reason = global::IpBanReason::WsTooManyReconnections;
        let ban_duration = TimeDelta::try_seconds(10).unwrap();
        let mut banned_until: Option<(DateTime<Utc>, global::IpBanReason)> = None;
        
        let mut block_tracker = global::ThresholdTracker::new(time);
        let mut ban_tracker = global::ThresholdTracker::new(time);
        let block_threshold = global::Threshold::new(10, TimeDelta::try_seconds(10).unwrap());
        let ban_threshold = global::Threshold::new(10, TimeDelta::try_seconds(10).unwrap());

        

        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(result, AllowCon::Allow);

        for _ in 0..15 {
            let result = double_throttle(
                &mut block_tracker,
                &mut ban_tracker,
                &block_threshold,
                &ban_threshold,
                ban_reason,
                &ban_duration,
                &time,
                &mut banned_until,
            );
        }

        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                block_tracker.amount,
                ban_tracker.amount
            ),
            (AllowCon::Blocked, 10, 7)
        );

        for _ in 0..3 {
            let result = double_throttle(
                &mut block_tracker,
                &mut ban_tracker,
                &block_threshold,
                &ban_threshold,
                ban_reason,
                &ban_duration,
                &time,
                &mut banned_until,
            );
        }

        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );
        assert_eq!(
            (
                result,
                block_tracker.amount,
                ban_tracker.amount
            ),
            (
                AllowCon::Banned((
                    time.checked_add_signed(ban_duration).unwrap(),
                    global::IpBanReason::WsTooManyReconnections
                )),
                10,
                10
            )
        );

        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                block_tracker.amount,
                ban_tracker.amount
            ),
            (AllowCon::AlreadyBanned,  10, 10)
        );

        let now = time.checked_add_signed(ban_duration).unwrap();
        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                block_tracker.amount,
                ban_tracker.amount
            ),
            (AllowCon::UnbannedAndAllow, 1, 0)
        );

        let result = double_throttle(
            &mut block_tracker,
            &mut ban_tracker,
            &block_threshold,
            &ban_threshold,
            ban_reason,
            &ban_duration,
            &time,
            &mut banned_until,
        );

        assert_eq!(
            (
                result,
                block_tracker.amount,
                ban_tracker.amount
            ),
            (AllowCon::Allow, 2, 0)
        );
    }

    #[test]
    fn threshold_tracker() {
        init_logger();

        let mut time = Utc::now();
        let max = 5;
        let delta = TimeDelta::try_seconds(5).unwrap();
        let mut tracker = global::ThresholdTracker::new(time);
        let threshold = global::Threshold::new(max, delta);
        
        let allow = threshold_allow(&mut tracker, &threshold, &time);
        assert!(allow);

        tracker.amount = max - 1;

        let allow = threshold_allow(&mut tracker, &threshold, &time);
        assert!(allow);

        tracker.amount = max;

        let allow = threshold_allow(&mut tracker, &threshold, &time);
        assert!(!allow);

        tracker.amount = max - 1;

        let allow = threshold_allow(&mut tracker, &threshold, &time);
        assert!(allow);

        tracker.amount = max;
        time += delta;

        let allow = threshold_allow(&mut tracker, &threshold, &time);
        assert!(allow);
    }

    fn init_logger() {
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
