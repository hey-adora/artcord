use chrono::{DateTime, Utc};
use field_types::FieldName;
use leptos::RwSignal;
use serde::de::value;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum::VariantNames;
use std::num::TryFromIntError;
use std::sync::mpsc;
use std::{collections::HashMap, fmt::Debug, str::FromStr};
use tracing::error;
use thiserror::Error;

use crate::message::prod_client_msg::ClientMsgIndexType;
use crate::message::prod_client_msg::ClientMsg;



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

impl TryFrom<(ClientMsgIndexType, WsStatTempCountItem)> for ReqCount {
    type Error = ReqCountTryFromError;

    fn try_from((msg_enum_index, connection_count): (ClientMsgIndexType, WsStatTempCountItem)) -> Result<Self, Self::Error> {
        let path = ClientMsg::VARIANTS.get(msg_enum_index).ok_or(ReqCountTryFromError::InvalidClientMsgEnumIndex(msg_enum_index))?;
        let count = i64::try_from(connection_count.total_count)?;
        Ok(
            Self {
                count,
                path: path.to_string(),
            }
        )
    }
}



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
pub struct WsStatDb {
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

impl WsStatDb {
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

    pub fn from_hashmap_temp_stats(temp_stats: HashMap<TempConIdType, WsStatTemp>) -> Result<Vec<Self>, ReqCountTryFromError> {
        temp_stats.into_iter().map(|(connection_uuid, connection_temp_stats)| WsStatDb::from_temp(connection_temp_stats, uuid::Uuid::from_u128(connection_uuid).to_string(), Utc::now().timestamp_millis())).collect()
    }
}

// impl TryFrom<WsStatTemp> for WsStatDb {
//     type Error = ReqCountTryFromError;

//     fn try_from(value: WsStatTemp) -> Result<Self, Self::Error> {
//         Self::from_temp(value, Utc::now().timestamp_millis())
//     }
// }

impl WsStatDb {
    pub fn from_temp(value: WsStatTemp, id: String, disconnected_at: i64) -> Result<Self, ReqCountTryFromError> {
        let req_count: Vec<ReqCount> = value.count.into_iter().map(|v| v.try_into()).collect::<Result<Vec<ReqCount>, ReqCountTryFromError>>()?;

        Ok(
            Self {
                id,
                ip: value.ip,
                addr: value.addr,
                req_count,
                connected_at: value.connected_at,
                disconnected_at: disconnected_at,
                modified_at: Utc::now().timestamp_millis(),
                created_at: Utc::now().timestamp_millis(),
            }
        )
    }
}

pub type TempConIdType = u128;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct WsStatTempCountItem {
    pub total_count: u64,
    pub count: u64,
    pub last_reset_at: i64,
}

impl Default for WsStatTempCountItem {
    fn default() -> Self {
        Self {
            total_count: 0,
            count: 0,
            last_reset_at: Utc::now().timestamp_millis()
        }
    }
}

impl WsStatTempCountItem {
    pub fn new(total_count: u64) -> Self {
        Self {
            total_count,
            ..Self::default()
        }
    }
}

pub type AdminStatCountType = HashMap<ClientMsgIndexType, WsStatTempCountItem>;

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

impl TryFrom<WsStatDb> for WsStatTemp {
    type Error = WsStatDbToTempTryFromError;

    fn try_from(value: WsStatDb) -> Result<WsStatTemp, Self::Error> {
        let mut count = HashMap::<ClientMsgIndexType, WsStatTempCountItem>::with_capacity(value.req_count.len());
        for req_count in value.req_count {
            let client_msg_enum_index = ClientMsg::VARIANTS.iter().position(|name| *name == req_count.path).ok_or(WsStatDbToTempTryFromError::InvalidClientMsgEnumName(req_count.path))?;
            let total_count = u64::try_from(req_count.count)?;
            let count_item = WsStatTempCountItem::new(total_count);

            count.insert(
                client_msg_enum_index,
                count_item,
            );
        }

        Ok(
            Self {
                ip: value.ip,
                addr: value.addr,
                count,
                connected_at: value.connected_at,
            }
        )
    }
}


pub type WebAdminStatCountType = HashMap<ClientMsgIndexType, RwSignal<u64>>;

#[derive(Debug, PartialEq, Clone)]
pub struct WebWsStat {
    pub addr: String,
    // pub is_connected: RwSignal<bool>,
    pub count: WebAdminStatCountType,
}

impl From<WsStatTemp> for WebWsStat {
    fn from(value: WsStatTemp) -> Self {

        //let a: AdminStatCountType = value.count.iter().map(|a| 0).collect();
        let count_map = value.count.iter().fold(WebAdminStatCountType::new(), |mut prev, (key, value)| {
            prev.insert(*key, RwSignal::new(value.total_count));
            prev
        });
        // let mut count_map: WebAdminStatCountType = HashMap::with_capacity(value.count.len());
        // for path in WsPath::iter() {
        //     count_map.insert(
        //         path,
        //         RwSignal::new(value.count.get(&path).cloned().unwrap_or(0_u64)),
        //     );
        // }
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


#[derive(Error, Debug)]
pub enum ReqCountTryFromError {
    #[error("Failed to convert u64 to i64: {0}")]
    TryFromIntError(#[from] TryFromIntError),

    #[error("Invalid client msg enum index - out of bounds: {0}")]
    InvalidClientMsgEnumIndex(usize),
}

#[derive(Error, Debug)]
pub enum WsStatDbToTempTryFromError {
    #[error("Failed to convert i64 to u64: {0}")]
    TryFromIntError(#[from] TryFromIntError),

    #[error("Invalid client msg enum name - name not found: {0}")]
    InvalidClientMsgEnumName(String),
}