#[cfg(feature = "ssr")]
pub mod bot;
//mod queries;

use bson::oid::ObjectId;
use bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllowedRole {
    pub _id: ObjectId,
    pub guild_id: String,
    pub id: String,
    pub name: String,
    pub feature: String,
    pub modified_at: DateTime,
    pub created_at: DateTime,
}
