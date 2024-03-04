use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConnectionWs {
    pub id: String,
    pub ip: String,
    pub agent: String,
    pub acc: String,
    pub last_used: i64,
    pub modified_at: i64,
    pub created_at: i64,
}
