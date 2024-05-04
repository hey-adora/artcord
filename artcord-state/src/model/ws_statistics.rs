use chrono::Utc;
use field_types::FieldName;
use leptos::RwSignal;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use std::{collections::HashMap, fmt::Debug, str::FromStr};
use tracing::error;

use crate::message::{prod_client_msg::WsPath};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ReqCount {
    pub path: String,
    pub count: i64,
}

impl ReqCount {
    pub fn new(path: String, count: i64) -> Self {
        Self { path, count }
    }
}

impl From<(WsPath, u64)> for ReqCount {
    fn from(value: (WsPath, u64)) -> Self {
        let count = i64::try_from(value.1)
            .inspect_err(|e| error!("ws_stats overflow {}", e))
            .unwrap_or(0);

        let path: &'static str = value.0.into();

        Self {
            count,
            path: path.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
pub struct WsStat {
    pub id: String,
    pub ip: String,
    pub addr: String,
    //pub is_connected: bool,
    // pub addr: String,
    pub req_count: Vec<ReqCount>,
    // pub req_count_main_gallery: i64,
    // pub req_count_user_gallery: i64,
    // pub agent: String,
    // pub acc: String,
    // pub last_used: i64,
    pub connected_at: i64,
    pub disconnected_at: i64,
    pub modified_at: i64,
    pub created_at: i64,
}
// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
// pub struct Statistic {
//     pub id: String,
//     pub ip: String,
//     pub is_connected: bool,
//     pub addr: String,
//     pub req_count_main_gallery: i64,
//     pub req_count_user_gallery: i64,
//     // pub agent: String,
//     // pub acc: String,
//     // pub last_used: i64,
//     pub modified_at: i64,
//     pub created_at: i64,
// }

impl WsStat {
    pub fn new(ip: String, addr: String, connected_at: i64, disconnected_at: i64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            ip,
            addr,
            req_count: vec![],
            connected_at,
            disconnected_at,
            modified_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
        }
    }

    pub fn from_hashmap_temp_stats(temp_stats: HashMap<String, WsStatTemp>) -> Vec<Self> {
        temp_stats.into_iter().map(|v| v.1.into()).collect()
    }
}

impl From<WsStatTemp> for WsStat {
    fn from(value: WsStatTemp) -> Self {
        Self::from_temp(value, Utc::now().timestamp_millis())
    }
}

impl WsStat {
    fn from_temp(value: WsStatTemp, disconnected_at: i64) -> Self {
        let req_count: Vec<ReqCount> = value.count.into_iter().map(|v| v.into()).collect();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            ip: value.ip,
            addr: value.addr,
            req_count,
            connected_at: value.connected_at,
            disconnected_at: disconnected_at,
            modified_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
        }
    }
}


pub type AdminStatCountType = HashMap<WsPath, u64>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct WsStatTemp {
    pub ip: String,
    pub addr: String,
    pub count: AdminStatCountType,
    pub connected_at: i64,
}

impl WsStatTemp {
    pub fn new(ip: String, addr: String, connected_at: i64) -> Self {
        Self {
            ip,
            addr,
            count: HashMap::new(),
            connected_at,
        }
    }
}

impl From<WsStat> for WsStatTemp {
    fn from(value: WsStat) -> Self {
        let mut count = HashMap::<WsPath, u64>::with_capacity(value.req_count.len());
        for req_count in value.req_count {
            count.insert(
                WsPath::from_str(&req_count.path)
                    .inspect_err(|e| error!("ws_stat_temp invalid path: {}", e))
                    .unwrap_or(WsPath::WsStatsPaged),
                req_count.count as u64,
            );
        }

        Self {
            ip: value.ip,
            addr: value.addr,
            count,
            connected_at: value.connected_at,
        }
    }
}


pub type WebAdminStatCountType = HashMap<WsPath, RwSignal<u64>>;

#[derive(Debug, PartialEq, Clone)]
pub struct WebWsStat {
    pub addr: String,
    // pub is_connected: RwSignal<bool>,
    pub count: WebAdminStatCountType,
}

impl From<WsStatTemp> for WebWsStat {
    fn from(value: WsStatTemp) -> Self {
        let mut count_map: WebAdminStatCountType = HashMap::with_capacity(value.count.len());
        for path in WsPath::iter() {
            count_map.insert(
                path,
                RwSignal::new(value.count.get(&path).cloned().unwrap_or(0_u64)),
            );
        }
        // for (path, count) in value.count {
        //     count_map.insert(path, RwSignal::new(count));
        // }
        WebWsStat {
            addr: value.addr,
            // is_connected: RwSignal::new(true),
            count: count_map,
        }
    }
}