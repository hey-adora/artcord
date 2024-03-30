use chrono::Utc;
use field_types::FieldName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
pub struct AllowedGuild {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    pub modified_at: i64,
    pub created_at: i64,
}

impl AllowedGuild {
    pub fn new(guild_id: String, name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            guild_id,
            name,
            created_at: Utc::now().timestamp_millis(),
            modified_at: Utc::now().timestamp_millis(),
        }
    }
}
