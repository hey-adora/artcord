use field_types::FieldName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
pub struct Migration {
    pub name: String,
    pub version: u32,
    pub modified_at: i64,
    pub created_at: i64,
}
