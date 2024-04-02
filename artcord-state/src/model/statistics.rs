use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Statistic {
    pub id: String,
    pub ip: String,
    // pub agent: String,
    // pub acc: String,
    // pub last_used: i64,
    pub modified_at: i64,
    pub created_at: i64,
}

impl Statistic {
    pub fn new(ip: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            ip,
            modified_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
        }
    }
}
