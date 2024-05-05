use chrono::Utc;
use field_types::FieldName;
use leptos::RwSignal;
use serde::de::value;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum::VariantNames;
use std::num::TryFromIntError;
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

impl TryFrom<(ClientMsgIndexType, u64)> for ReqCount {
    type Error = ReqCountTryFromError;

    fn try_from((msg_enum_index, connection_count): (ClientMsgIndexType, u64)) -> Result<Self, Self::Error> {
        let path = ClientMsg::VARIANTS.get(msg_enum_index).ok_or(ReqCountTryFromError::InvalidClientMsgEnumIndex(msg_enum_index))?;
        let count = i64::try_from(connection_count)?;
        Ok(
            Self {
                count,
                path: path.to_string(),
            }
        )
    }
}


#[derive(Error, Debug)]
pub enum ReqCountTryFromError {
    #[error("Failed to convert u64 to i64: {0}")]
    TryFromIntError(#[from] TryFromIntError),

    #[error("Invalid client msg enum index - out of bounds: {0}")]
    InvalidClientMsgEnumIndex(usize),
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

    pub fn from_hashmap_temp_stats(temp_stats: HashMap<String, WsStatTemp>) -> Result<Vec<Self>, ReqCountTryFromError> {
        temp_stats.into_iter().map(|(_connection_uuid, connection_temp_stats)| connection_temp_stats.try_into()).collect()
    }
}

impl TryFrom<WsStatTemp> for WsStatDb {
    type Error = ReqCountTryFromError;

    fn try_from(value: WsStatTemp) -> Result<Self, Self::Error> {
        Self::from_temp(value, Utc::now().timestamp_millis())
    }
}

impl WsStatDb {
    fn from_temp(value: WsStatTemp, disconnected_at: i64) -> Result<Self, ReqCountTryFromError> {
        let req_count: Vec<ReqCount> = value.count.into_iter().map(|v| v.try_into()).collect::<Result<Vec<ReqCount>, ReqCountTryFromError>>()?;

        Ok(
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
        )
    }
}


pub type AdminStatCountType = HashMap<ClientMsgIndexType, u64>;

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
    type Error = ();

    fn try_from(value: WsStatDb) -> Result<WsStatTemp, Self::Error> {
        let mut count = HashMap::<ClientMsgIndexType, u64>::with_capacity(value.req_count.len());
        for req_count in value.req_count {
            let client_msg_enum_index = ClientMsg::VARIANTS.iter().position(|name| *name == req_count.path);
            let Some(client_msg_enum_index) = client_msg_enum_index else {
                error!("failed to convert {:?} to WsStatTemp, invalid variant name", &req_count);
                return Err(());
            };

            
            count.insert(
                client_msg_enum_index,
                req_count.count as u64,
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
            prev.insert(*key, RwSignal::new(*value));
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