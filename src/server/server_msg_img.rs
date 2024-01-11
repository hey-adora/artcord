use crate::bot::img_quality::ImgQuality;
use crate::database::models::user::User;
use crate::database::rkw::date_time::DT;
use crate::database::rkw::object_id::OBJ;
use bson::oid::ObjectId;
use bson::DateTime;
use chrono::Utc;

#[derive(
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    Debug,
    PartialEq,
    Clone,
    serde::Serialize,
    serde::Deserialize,
)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub struct ServerMsgImg {
    #[with(OBJ)]
    pub _id: ObjectId,
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

    #[with(DT)]
    pub modified_at: bson::datetime::DateTime,

    #[with(DT)]
    pub created_at: bson::datetime::DateTime,
}

impl Default for ServerMsgImg {
    fn default() -> Self {
        Self {
            _id: ObjectId::new(),
            user: User {
                _id: ObjectId::new(),
                guild_id: String::from("1159766826620817419"),
                id: String::from("id"),
                name: String::from("name"),
                pfp_hash: Some(String::from("pfp_hash")),
                modified_at: DateTime::from_millis(Utc::now().timestamp_millis()),
                created_at: DateTime::from_millis(Utc::now().timestamp_millis()),
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
            modified_at: DateTime::from_millis(Utc::now().timestamp_millis()),
            created_at: DateTime::from_millis(Utc::now().timestamp_millis()),
        }
    }
}

impl ServerMsgImg {
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
