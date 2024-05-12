use chrono::{TimeDelta, Utc};
use chrono::{DateTime, Days};
use leptos::RwSignal;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::{collections::HashMap, time::Instant};
use tracing::{error, trace, warn};

use crate::message::prod_client_msg::ClientMsgIndexType;
use crate::util::time::time_is_past;

use super::throttle_threshold::ThrottleRanged;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
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
    pub ws_path_count: HashMap<ClientMsgIndexType, LiveThrottleConnectionCount>,
    pub throttle: ThrottleRanged,
}

impl LiveThrottleConnection {
    pub fn new(range: u64, time: DateTime<Utc>) -> Self {
        Self {
            ws_path_count: HashMap::new(),
            throttle: ThrottleRanged::new(range, time)
        }
    }


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
            ws_connection_count: RwSignal::new(value.throttle.amount),
            ws_path_count: RwSignal::new(WebThrottleConnectionCount::from_live(
                value.ws_path_count,
            )),
            ws_total_blocked_connection_attempts: RwSignal::new(
                value.throttle.tracker.total_amount,
            ),
            ws_blocked_connection_attempts: RwSignal::new(value.throttle.tracker.total_amount),
            ws_blocked_connection_attempts_last_reset_at: RwSignal::new(
                value.throttle.tracker.started_at,
            ),
            ws_banned_until: RwSignal::new(value.throttle.banned_until),
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
