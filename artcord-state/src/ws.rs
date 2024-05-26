use std::net::IpAddr;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use chrono::DateTime;
use crate::misc::throttle_connection::IpBanReason;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct WsIpStat {
    pub ip: IpAddr,
    pub total_allow_amount: u64,
    pub total_block_amount: u64,
    pub total_banned_amount: u64,
    pub total_already_banned_amount: u64,
    //pub total_unbanned_amount: u64,
    pub banned_until: Option<(DateTime<Utc>, IpBanReason)>,
}

impl WsIpStat {
    pub fn new(ip: IpAddr) -> Self {
        Self {
            ip,
            total_allow_amount: 0,
            total_block_amount: 0,
            total_banned_amount: 0,
            total_already_banned_amount: 0,
            //total_unbanned_amount: 0,
            banned_until: None,
        }
    }
}