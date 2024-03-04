// #[cfg(feature = "ssr")]
// pub mod bot;

// use bson::oid::ObjectId;
// use bson::DateTime;
use field_types::FieldName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
pub struct AllowedChannel {
    pub guild_id: String,
    pub name: String,
    pub channel_id: String,
    pub feature: String,
    pub modified_at: i64,
    pub created_at: i64,
}
