use chrono::Utc;
use field_types::FieldName;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};
use tracing::error;

use crate::message::{prod_client_msg::WsPath, prod_server_msg::WsStatTemp};

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
    pub addr: String,
    pub is_connected: bool,
    // pub addr: String,
    pub req_count: Vec<ReqCount>,
    // pub req_count_main_gallery: i64,
    // pub req_count_user_gallery: i64,
    // pub agent: String,
    // pub acc: String,
    // pub last_used: i64,
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
    pub fn new(addr: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            // ip,
            addr,
            is_connected: false,
            req_count: vec![],
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
        let req_count: Vec<ReqCount> = value.count.into_iter().map(|v| v.into()).collect();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            addr: value.addr,
            is_connected: true,
            req_count,
            modified_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
        }
    }
}

// impl From<> for Vec<WsStatistic> {
//     fn from(value: HashMap<WsStatisticTemp>) -> Self {
//
//     }
// }
