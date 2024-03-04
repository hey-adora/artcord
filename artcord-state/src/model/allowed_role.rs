// #[cfg(feature = "ssr")]
// pub mod bot;
// //mod queries;

// use bson::oid::ObjectId;
// use bson::DateTime;
use field_types::FieldName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
pub struct AllowedRole {
    pub role_id: String,
    pub guild_id: String,
    pub name: String,
    pub feature: String,
    pub modified_at: i64,
    pub created_at: i64,
}
