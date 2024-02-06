use chrono::Utc;
use field_types::FieldName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
pub struct AllowedGuild {
    pub guild_id: String,
    pub name: String,
    pub modified_at: i64,
    pub created_at: i64,
}

impl AllowedGuild {
    pub fn new(id: String, name: String) -> Self {
        Self {
            guild_id: id,
            name,
            created_at: Utc::now().timestamp_millis(),
            modified_at: Utc::now().timestamp_millis(),
        }
    }
}
