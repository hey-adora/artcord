// use crate::message::server_msg::ServerMsg;
use bson::{oid::ObjectId, Uuid};
use chrono::Utc;
use field_types::FieldName;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;



impl AccSession {
    pub fn new(acc_id: String, ip: String, agent: String, token: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            acc_id,
            ip,
            agent,
            token,
            last_used: Utc::now().timestamp_millis(),
            modified_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
        }
    }
}
