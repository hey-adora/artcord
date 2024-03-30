// use crate::img_quality::ImgQuality;
// use bson::oid::ObjectId;
// use bson::DateTime;
use field_types::FieldName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, FieldName)]
pub struct Img {
    pub id: String,
    pub msg_id: String,
    pub show: bool,
    pub guild_id: String,
    pub user_id: String,
    pub channel_id: String,
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

// impl Img {
//     pub fn pick_quality(&self) -> ImgQuality {
//         if self.has_high {
//             ImgQuality::High
//         } else if self.has_medium {
//             ImgQuality::Medium
//         } else if self.has_low {
//             ImgQuality::Low
//         } else {
//             ImgQuality::Org
//         }
//     }
// }
