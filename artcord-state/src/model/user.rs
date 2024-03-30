use field_types::FieldName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
pub struct User {
    pub id: String,
    pub author_id: String,
    pub guild_id: String,
    pub name: String,
    pub pfp_hash: Option<String>,
    pub modified_at: i64,
    pub created_at: i64,
}
