// #[cfg(feature = "ssr")]
// pub mod bot;
// //mod queries;

use chrono::Utc;
// use bson::oid::ObjectId;
// use bson::DateTime;
use field_types::FieldName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
pub struct AllowedRole {
    pub id: String,
    pub role_id: String,
    pub guild_id: String,
    pub name: String,
    pub feature: String,
    pub modified_at: i64,
    pub created_at: i64,
}

impl AllowedRole {
    pub fn new(role_id: String, guild_id: String, name: String, feature: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role_id,
            guild_id,
            name,
            feature,
            created_at: Utc::now().timestamp_millis(),
            modified_at: Utc::now().timestamp_millis(),
        }
    }
}
