use bson::DateTime;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::bot::img_quality::ImgQuality;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Img {
    pub _id: ObjectId,
    pub show: bool,
    pub guild_id: String,
    pub user_id: String,
    pub channel_id: String,
    pub id: String,
    pub org_url: String,
    pub org_hash: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub has_high: bool,
    pub has_medium: bool,
    pub has_low: bool,
    pub modified_at: DateTime,
    pub created_at: DateTime,
}

impl Img {
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