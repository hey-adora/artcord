// #[cfg(feature = "ssr")]
// pub mod serd;

use chrono::Utc;
//use crate::img_quality::ImgQuality;
// use bson::oid::ObjectId;
// use bson::DateTime;
use field_types::FieldName;
use serde::{Deserialize, Serialize};
use std::num::ParseIntError;

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, FieldName)]
pub struct AutoReaction {
    pub id: String,
    pub emoji_id: Option<String>,
    pub guild_id: String,
    pub unicode: Option<String>,
    pub name: Option<String>,
    pub animated: bool,
    pub modified_at: i64,
    pub created_at: i64,
}

impl AutoReaction {
    pub fn new(
        guild_id: String,
        unicode: Option<String>,
        emoji_id: Option<String>,
        name: Option<String>,
        animated: bool,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            guild_id,
            unicode,
            emoji_id,
            name,
            animated,
            modified_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
        }
    }
}
