// use crate::bot::img_quality::ImgQuality;
// use crate::database::models::user::User;
use chrono::Utc;
use field_types::FieldName;
use serde::{Deserialize, Serialize};

use crate::{misc::img_quality::ImgQuality, model::user::User};

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, FieldName)]
pub struct AggImg {
    pub id: String,
    pub user: User,
    pub user_id: String,
    pub org_url: String,
    pub org_hash: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub has_high: bool,
    pub has_medium: bool,
    pub has_low: bool,
    pub modified_at: i64,
    pub created_at: i64,
}

impl Default for AggImg {
    fn default() -> Self {
        Self {
            user: User {
                id: String::from("id"),
                guild_id: String::from("1159766826620817419"),
                name: String::from("name"),
                pfp_hash: Some(String::from("pfp_hash")),
                modified_at: Utc::now().timestamp_millis(),
                created_at: Utc::now().timestamp_millis(),
            },
            org_url: String::from("wow"),
            user_id: String::from("1159037321283375174"),
            id: String::from("1177244237021073450"),
            org_hash: String::from("2552bd2db66978a9b3675721e95d1cbd"),
            format: String::from("png"),
            width: 233,
            height: 161,
            has_high: false,
            has_medium: false,
            has_low: false,
            modified_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
        }
    }
}

impl AggImg {
    pub fn pick_quality(&self) -> ImgQuality {
        if self.has_high {
            ImgQuality::High
        } else if self.has_medium {
            ImgQuality::Medium
        } else if self.has_low {
            ImgQuality::Low
        } else {
            ImgQuality::Org
        }
    }
}
