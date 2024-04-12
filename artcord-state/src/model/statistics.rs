use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ReqCount {
    pub path: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Statistic {
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

impl Statistic {
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
}
