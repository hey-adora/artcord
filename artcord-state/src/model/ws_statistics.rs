use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use field_types::FieldName;
use leptos::RwSignal;
use serde::de::value;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum::VariantNames;
use std::net::IpAddr;
use std::num::TryFromIntError;
use std::sync::mpsc;
use std::{collections::HashMap, fmt::Debug, str::FromStr};
use tracing::error;
use thiserror::Error;
use std::net::SocketAddr;

use crate::message::prod_client_msg::ClientPathType;
use crate::message::prod_client_msg::ClientMsg;
use crate::misc::throttle_connection::IpBanReason;
use crate::misc::throttle_threshold::{AllowCon, DbThrottleDoubleLayer, DbThrottleDoubleLayerFromError, Threshold, ThrottleDoubleLayer, ThrottleDoubleLayerFromError};

pub type TempConIdType = u128;



#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct WsStat {
    pub con_id: TempConIdType,
    pub ip: IpAddr,
    pub addr: SocketAddr,
    pub count:  HashMap<ClientPathType, WsStatPath>,
    pub connected_at: DateTime<Utc>,
    pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
    //pub throttle: ThrottleDoubleLayer,
}



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
pub struct DbWsStat {
    pub id: String,
    pub con_id: String,
    pub ip: String,
    pub addr: String,
    pub req_count: Vec<DbWsStatPath>,
    pub connected_at: i64,
    pub disconnected_at: i64,
    //pub throttle: DbThrottleDoubleLayer,
    pub modified_at: i64,
    pub created_at: i64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct WsStatPath {
    // pub total_count: u64,
    // pub count: u64,
    pub total_allowed_count: u64,
    pub total_blocked_count: u64,
    pub total_banned_count: u64,
    pub total_already_banned_count: u64,
    //pub total_unbanned_count: u64,
    pub last_reset_at: DateTime<Utc>,
    pub throttle: ThrottleDoubleLayer,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct DbWsStatPath {
    pub path: String,
    pub total_allowed_count: i64,
    pub total_blocked_count: i64,
    pub total_banned_count: i64,
    pub total_already_banned_count: i64,
    //pub total_unbanned_count: i64,
    pub last_reset_at: i64,
    pub throttle: DbThrottleDoubleLayer,
}


// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
// pub struct ReqCount {
//     pub path: String,
//     pub count: i64,
// }

// impl ReqCount {
//     pub fn new(path: String, count: i64) -> Self {
//         Self { path, count }
//     }
// }

// impl TryFrom<(ClientMsgIndexType, WsStatTempCountItem)> for ReqCount {
//     type Error = ReqCountTryFromError;

//     fn try_from((msg_enum_index, connection_count): (ClientMsgIndexType, WsStatTempCountItem)) -> Result<Self, Self::Error> {
//         let path = ClientMsg::VARIANTS.get(msg_enum_index).ok_or(ReqCountTryFromError::InvalidClientMsgEnumIndex(msg_enum_index))?;
//         let count = i64::try_from(connection_count.total_count)?;
//         Ok(
//             Self {
//                 count,
//                 path: path.to_string(),
//             }
//         )
//     }
// }

impl WsStatPath {
    pub fn new(time: DateTime<Utc>) -> Self {
        Self {
            // total_count: 0,
            // count: 0,
            last_reset_at: time,
            total_allowed_count: 0,
            total_blocked_count: 0,
            total_banned_count: 0,
            total_already_banned_count: 0,
            //total_unbanned_count: 0,
            throttle: ThrottleDoubleLayer::new(time)
        }
    }

    
}

impl DbWsStat {
    pub fn from_hashmap_ws_stats(temp_stats: HashMap<TempConIdType, WsStat>, time: DateTime<Utc>) -> Result<Vec<Self>, DbWsStatPathFromError> {
        temp_stats.into_iter().map(|(con_id, connection_temp_stats)| DbWsStat::from_ws_stat(connection_temp_stats, uuid::Uuid::from_u128(con_id).to_string(), time, time)).collect()
    }

    pub fn from_ws_stat(value: WsStat, con_id: String, disconnected_at: DateTime<Utc>, time: DateTime<Utc>) -> Result<Self, DbWsStatPathFromError> {
        let req_count: Vec<DbWsStatPath> = value.count.into_iter().map(|v| v.try_into()).collect::<Result<Vec<DbWsStatPath>, DbWsStatPathFromError>>()?;

        Ok(
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                con_id,
                ip: value.ip.to_string(),
                addr: value.addr.to_string(),
                req_count,
                connected_at: value.connected_at.timestamp_millis(),
                disconnected_at: disconnected_at.timestamp_millis(),
                //throttle: value.throttle.try_into()?,
                modified_at: time.timestamp_millis(),
                created_at: time.timestamp_millis(),
            }
        )
    }
}


/////////////////////////////

impl TryFrom<(ClientPathType, WsStatPath)> for DbWsStatPath {
    type Error = DbWsStatPathFromError;
    fn try_from((path, value): (ClientPathType, WsStatPath)) -> Result<Self, Self::Error> {
        let path = ClientMsg::VARIANTS.get(path).ok_or(DbWsStatPathFromError::InvalidClientMsgEnumIndex(path))?;
        Ok(
            Self {
                path: path.to_string(),
                total_allowed_count: i64::try_from(value.total_allowed_count)?,
                total_blocked_count: i64::try_from(value.total_blocked_count)?,
                total_banned_count: i64::try_from(value.total_banned_count)?,
                total_already_banned_count: i64::try_from(value.total_already_banned_count)?,
                //total_unbanned_count: i64::try_from(value.total_unbanned_count)?,
                // total_count: i64::try_from(value.total_count)?,
                // count: i64::try_from(value.count)?,
                // total_count: i64::try_from(value.count)?
                last_reset_at: value.last_reset_at.timestamp_millis(),
                throttle: value.throttle.try_into()?,
            }
        )
    }
}

impl TryFrom<DbWsStatPath> for WsStatPath {
    type Error = DbWsStatTempCountItemError;
    fn try_from(value: DbWsStatPath) -> Result<Self, Self::Error> {
        Ok(
            Self {
                // total_count: u64::try_from(value.total_count)?,
                // count: u64::try_from(value.count)?,
                total_allowed_count: u64::try_from(value.total_allowed_count)?,
                total_blocked_count: u64::try_from(value.total_blocked_count)?,
                total_banned_count: u64::try_from(value.total_banned_count)?,
                total_already_banned_count: u64::try_from(value.total_already_banned_count)?,
                //total_unbanned_count: u64::try_from(value.total_unbanned_count)?,

                throttle: value.throttle.try_into()?,
                last_reset_at: DateTime::<Utc>::from_timestamp_millis(value.last_reset_at).ok_or(DbWsStatTempCountItemError::InvalidDate(value.last_reset_at))?,
            }
        )
    }
}

impl WsStat {
    pub fn new(con_id: TempConIdType , ip: IpAddr, addr: SocketAddr, started_at: DateTime<Utc>) -> Self {
        Self {
            con_id,
            ip,
            addr,
            count: HashMap::new(),
            connected_at: started_at,
            banned_until: None,
            //throttle: ThrottleDoubleLayer::new(started_at),
        }
    }

    pub async fn inc_path(
        &mut self,
        path: ClientPathType,
        block_threshold: Threshold,
        ban_threshold: &Threshold,
        ban_duration: &TimeDelta,
       // banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
        time: &DateTime<Utc>,
    ) -> AllowCon {
        let path = self
            .count
            .entry(path)
            .or_insert_with(|| WsStatPath::new(*time));

        let result = path.throttle.allow(
            &block_threshold,
            ban_threshold,
            IpBanReason::WsRouteBruteForceDetected,
            ban_duration,
            time,
            &mut self.banned_until,
        );

        //path.total_count += 1;

        match &result {
            AllowCon::Allow => {
                path.total_allowed_count += 1;
            }
            AllowCon::Unbanned => {
                path.total_allowed_count += 1;
            }
            AllowCon::Blocked => {
                path.total_blocked_count += 1;
            }
            AllowCon::Banned(_) => {
                path.total_banned_count += 1;
            }
            AllowCon::AlreadyBanned => {
                path.total_already_banned_count += 1;
            }
        }

        result
    }
}

impl TryFrom<DbWsStat> for WsStat {
    type Error = WsStatDbToTempTryFromError;

    fn try_from(value: DbWsStat) -> Result<WsStat, Self::Error> {
        let mut count = HashMap::<ClientPathType, WsStatPath>::with_capacity(value.req_count.len());
        for req_count in value.req_count {
            let client_msg_enum_index = ClientMsg::VARIANTS.iter().position(|name| *name == req_count.path).ok_or(WsStatDbToTempTryFromError::InvalidClientMsgEnumName(req_count.path.clone()))?;
            //let total_count = u64::try_from(req_count.count)?;
            //let count_item = WsStatPath::from_db(total_count);

            count.insert(
                client_msg_enum_index,
                req_count.try_into()?,
            );
        }

        let con_id = uuid::Uuid::from_str(&value.con_id)?;

        Ok(
            Self {
                con_id: con_id.as_u128(),
                ip: IpAddr::from_str(&value.ip)?,
                addr: SocketAddr::from_str(&value.addr)?,
                count,
                //throttle: value.throttle.try_into()?,
                connected_at: DateTime::<Utc>::from_timestamp_millis(value.connected_at).ok_or(WsStatDbToTempTryFromError::InvalidDate(value.connected_at))?,
                banned_until: None,
            }
        )
    }
}



#[derive(Error, Debug)]
pub enum DbWsStatTempCountItemError {
    #[error("Failed to convert i64 to u64: {0}")]
    TryFromIntError(#[from] TryFromIntError),
    
    #[error("error converting double_layer_throttle: {0}")]
    DoubleLayer(#[from] DbThrottleDoubleLayerFromError),

    #[error("invalid date: {0}")]
    InvalidDate(i64),
}


#[derive(Error, Debug)]
pub enum DbWsStatPathFromError {
    #[error("Failed to convert u64 to i64: {0}")]
    TryFromIntError(#[from] TryFromIntError),

    #[error("Invalid client msg enum index - out of bounds: {0}")]
    InvalidClientMsgEnumIndex(usize),

    #[error("error converting double_layer_throttle: {0}")]
    DoubleLayer(#[from] ThrottleDoubleLayerFromError),
}

#[derive(Error, Debug)]
pub enum WsStatDbToTempTryFromError {
    #[error("failed to parse string to socket_addr: {0}")]
    InvalidSocketAddr(#[from] std::net::AddrParseError),

    #[error("failed to convert path from database: {0}")]
    DbWsStatTempCountItem(#[from] DbWsStatTempCountItemError),
    
    #[error("failed to convert from database: {0}")]
    DbThrottleDoubleLayer(#[from] DbThrottleDoubleLayerFromError),
    
    #[error("Failed to convert i64 to u64: {0}")]
    TryFromIntError(#[from] TryFromIntError),

    #[error("Invalid client msg enum name - name not found: {0}")]
    InvalidClientMsgEnumName(String),

    #[error("Invalid date: {0}")]
    InvalidDate(i64),

    #[error("Invalid uuid: {0}")]
    InvalidUuid(#[from] uuid::Error),
}