use chrono::{TimeDelta, Utc};
use chrono::{DateTime, Days};
use leptos::RwSignal;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::{collections::HashMap, time::Instant};
use tracing::{error, trace, warn};

use crate::message::prod_client_msg::ClientMsgIndexType;
use crate::util::time::time_is_past;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum IpBanReason {
    TooManyConnectionAttempts,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct LiveThrottleConnectionCount {
    pub total_count: u64,
    pub count: u64,
    pub last_reset_at: DateTime<Utc>,
}

impl Default for LiveThrottleConnectionCount {
    fn default() -> Self {
        Self {
            total_count: 1,
            count: 1,
            last_reset_at: Utc::now(),
        }
    }
}

impl LiveThrottleConnectionCount {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct LiveThrottleConnection {
    pub ws_connection_count: u64,
    // wrap hashmap in SocketAddr (maybe)
    pub ws_path_count: HashMap<ClientMsgIndexType, LiveThrottleConnectionCount>,
    pub ws_total_blocked_connection_attempts: u64,
    pub ws_blocked_connection_attempts: u64,
    pub ws_blocked_connection_attempts_last_reset_at: DateTime<Utc>,
    pub ws_banned_until: Option<(DateTime<Utc>, IpBanReason)>,
    // ws_proccesing: RwLock<bool>,
    // ws_path_interval: RwLock<DateTime<chrono::Utc>>,
    //ws_last_connection: RwLock<u64>,
}

impl LiveThrottleConnection {
    pub fn new() -> Self {
        Self {
            ws_connection_count: 1,
            ws_path_count: HashMap::new(),
            ws_total_blocked_connection_attempts: 0,
            ws_blocked_connection_attempts: 0,
            ws_blocked_connection_attempts_last_reset_at: Utc::now(),
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

    // pub fn sync_with_web(&mut self, ips: &mut RwSignal<HashMap<IpAddr, WebThrottleConnection>>) {

    // }

    pub fn inc_path(&mut self, path: &ClientMsgIndexType) {
        let con_path = self.ws_path_count.get_mut(path);
        let Some(con_path) = con_path else {
            trace!("throttle: path inserted: {}", path);
            self.ws_path_count
                .insert(*path, LiveThrottleConnectionCount::new());
            return;
        };
        trace!(
            "throttle: path '{}' incremented from: {}, to: {}",
            path,
            con_path.total_count,
            con_path.total_count + 1
        );
        con_path.total_count += 1;
    }

    pub fn decrement_con_count(&mut self) -> bool {
        let val = self.ws_connection_count.checked_sub(1);
        let Some(val) = val else {
            error!("throttle: overflow prevented");
            return true;
        };
        trace!(
            "throttle: decremented con count from: {}, to: {}",
            self.ws_connection_count,
            val
        );
        self.ws_connection_count = val;

        self.ws_connection_count == 0
    }

    pub fn is_banned(&self) -> bool {
        let is_baned = self
            .ws_banned_until
            .as_ref()
            .map(|(until, _reason)| !time_is_past(until))
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

    pub fn ban(&mut self, reason: IpBanReason, ban_until_days: Days) -> Option<DateTime<Utc>> {
        trace!("throttle - banned: {:?}", &reason);
        let Some(date) = Utc::now().checked_add_days(ban_until_days) else {
            error!(
                "throtte: failed to ban, failed to add {:?} days to Utc::now()",
                ban_until_days
            );
            return None;
        };
        self.ws_banned_until = Some((date.clone(), reason));
        return Some(date);
    }

    pub fn inc_failed_total_con_attempts(&mut self) {
        trace!(
            "throttle - inc from: {} {} to {} {}",
            self.ws_total_blocked_connection_attempts,
            self.ws_blocked_connection_attempts,
            self.ws_total_blocked_connection_attempts + 1,
            self.ws_blocked_connection_attempts + 1
        );
        self.ws_total_blocked_connection_attempts += 1;
    }

    pub fn inc_failed_con_attempts(&mut self) {
        trace!(
            "throttle - inc from: {} {} to {} {}",
            self.ws_total_blocked_connection_attempts,
            self.ws_blocked_connection_attempts,
            self.ws_total_blocked_connection_attempts + 1,
            self.ws_blocked_connection_attempts + 1
        );
        self.ws_total_blocked_connection_attempts += 1;
        self.ws_blocked_connection_attempts += 1;
    }


    pub fn reached_max_failed_con_attempts_rate(&mut self, max_fail: u64, max_fail_rate: u64, max_delta: &TimeDelta) -> bool {
        trace!(
            "throttle - reached_max_rate check count: {} >= {} = {}",
            self.ws_blocked_connection_attempts,
            max_fail,
            self.ws_blocked_connection_attempts >= max_fail
        );
        if self.ws_blocked_connection_attempts >= max_fail {
            let time_passed = Utc::now() - self.ws_blocked_connection_attempts_last_reset_at;
            let rate = self
                .ws_blocked_connection_attempts
                .checked_div(time_passed.num_minutes() as u64)
                .unwrap_or(1);
            trace!(
                "throttle - reached_max_rate checking rate: {} >= {} = {} | {}",
                rate,
                max_fail,
                rate >= max_fail_rate,
                time_passed
            );
            let reached_max = rate >= max_fail_rate;

            if time_passed >= *max_delta {
                self.reset_max_failed_con_attempts_rate();
            }

            reached_max

        } else {
            false
        }
    }

    pub fn reset_max_failed_con_attempts_rate(&mut self) {
        let date = Utc::now();
        trace!(
            "throttle - reset from: {} {} to {} {}",
            self.ws_blocked_connection_attempts,
            self.ws_blocked_connection_attempts_last_reset_at,
            0,
            date
        );
        self.ws_blocked_connection_attempts = 0;
        self.ws_blocked_connection_attempts_last_reset_at = date;
    }

    pub fn check_con(
        &mut self,
        ip: &IpAddr,
        max_cons: u64,
        max_fail: u64,
        max_fail_rate: u64,
        ban_until_days: u64,
        max_delta: &TimeDelta,
    ) -> ConStatus {
        trace!(
            "ws({}): throttle: {} > {}",
            ip,
            self.ws_connection_count,
            max_cons
        );
        if self.ws_connection_count > max_cons {
            trace!(
                "ws({}): connection limit reached: {}",
                ip,
                self.ws_connection_count
            );
            self.inc_failed_con_attempts();
            if self.reached_max_failed_con_attempts_rate(max_fail, max_fail_rate, max_delta) {
                let reason = IpBanReason::TooManyConnectionAttempts;
                let banned = self.ban(
                    reason.clone(),
                    Days::new(ban_until_days),
                );
                //self.reset_max_failed_con_attempts_rate();
                return banned.map(|date| ConStatus::Banned((date, reason))).unwrap_or(ConStatus::Blocked(self.ws_total_blocked_connection_attempts, self.ws_blocked_connection_attempts));
            }
            return ConStatus::Blocked(self.ws_total_blocked_connection_attempts, self.ws_blocked_connection_attempts);
        }
        self.ws_connection_count += 1;
        trace!(
            "ws({}): throttle: incremented to: {}",
            ip,
            self.ws_connection_count
        );
        ConStatus::Allow
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ConStatus {
    Allow,
    Blocked(u64, u64),
    Banned((DateTime<Utc>, IpBanReason))
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct WebThrottleConnectionCount {
    pub total_count: RwSignal<u64>,
    pub count: RwSignal<u64>,
    pub last_reset_at: RwSignal<DateTime<Utc>>,
}

impl From<LiveThrottleConnectionCount> for WebThrottleConnectionCount {
    fn from(value: LiveThrottleConnectionCount) -> Self {
        Self {
            total_count: RwSignal::new(value.total_count),
            count: RwSignal::new(value.count),
            last_reset_at: RwSignal::new(value.last_reset_at),
        }
    }
}

impl Default for WebThrottleConnectionCount {
    fn default() -> Self {
        Self {
            total_count: RwSignal::new(1),
            count: RwSignal::new(1),
            last_reset_at: RwSignal::new(Utc::now()),
        }
    }
}

impl WebThrottleConnectionCount {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_live(
        value: HashMap<ClientMsgIndexType, LiveThrottleConnectionCount>,
    ) -> HashMap<ClientMsgIndexType, Self> {
        value
            .into_iter()
            .fold(HashMap::new(), |mut a, (key, value)| {
                a.insert(key, value.into());
                a
            })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct WebThrottleConnection {
    pub ws_connection_count: RwSignal<u64>,
    // wrap hashmap in SocketAddr (maybe)
    pub ws_path_count: RwSignal<HashMap<ClientMsgIndexType, WebThrottleConnectionCount>>,
    pub ws_total_blocked_connection_attempts: RwSignal<u64>,
    pub ws_blocked_connection_attempts: RwSignal<u64>,
    pub ws_blocked_connection_attempts_last_reset_at: RwSignal<DateTime<Utc>>,
    pub ws_banned_until: RwSignal<Option<(DateTime<Utc>, IpBanReason)>>,
    // ws_proccesing: RwLock<bool>,
    // ws_path_interval: RwLock<DateTime<chrono::Utc>>,
    //ws_last_connection: RwLock<u64>,
}

impl From<LiveThrottleConnection> for WebThrottleConnection {
    fn from(value: LiveThrottleConnection) -> Self {
        Self {
            ws_connection_count: RwSignal::new(value.ws_connection_count),
            ws_path_count: RwSignal::new(WebThrottleConnectionCount::from_live(
                value.ws_path_count,
            )),
            ws_total_blocked_connection_attempts: RwSignal::new(
                value.ws_total_blocked_connection_attempts,
            ),
            ws_blocked_connection_attempts: RwSignal::new(value.ws_blocked_connection_attempts),
            ws_blocked_connection_attempts_last_reset_at: RwSignal::new(
                value.ws_blocked_connection_attempts_last_reset_at,
            ),
            ws_banned_until: RwSignal::new(value.ws_banned_until),
        }
    }
}

impl Default for WebThrottleConnection {
    fn default() -> Self {
        Self {
            ws_connection_count: RwSignal::new(1),
            ws_path_count: RwSignal::new(HashMap::new()),
            ws_total_blocked_connection_attempts: RwSignal::new(0),
            ws_blocked_connection_attempts: RwSignal::new(0),
            ws_blocked_connection_attempts_last_reset_at: RwSignal::new(Utc::now()),
            ws_banned_until: RwSignal::new(None),
        }
    }
}

impl WebThrottleConnection {
    pub fn new() -> Self {
        Self::default()
    }


    pub fn from_live(
        value: HashMap<IpAddr, LiveThrottleConnection>,
    ) -> HashMap<IpAddr, WebThrottleConnection> {
        value
            .into_iter()
            .fold(HashMap::new(), |mut a, (key, value)| {
                a.insert(key, value.into());
                a
            })
    }
}
