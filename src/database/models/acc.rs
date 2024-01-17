use crate::database::rkw::date_time::DT;
use crate::database::rkw::object_id::OBJ;
use bson::oid::ObjectId;
use bson::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::server::server_msg::ServerMsg;

#[derive(
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Acc {
    #[with(OBJ)]
    pub _id: ObjectId,

    pub email: String,
    pub password: String,
    //pub salt: String,
    pub verified_email: bool,
    pub email_verification_code: String,

    pub discord: Option<AccDiscord>,

    #[with(DT)]
    pub modified_at: DateTime,

    #[with(DT)]
    pub created_at: DateTime,
}

#[derive(
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub struct AccDiscord {
    pub user_id: String,
    pub token: String,
}

impl Acc {
    pub fn new(
        email: &str,
        password: &str,
        email_verification_code: &str,
    ) -> Acc {
        Acc {
            _id: ObjectId::new(),
            email: email.to_string(),
            verified_email: false,
            email_verification_code: email_verification_code.to_string(),
            password: password.to_string(),
            //salt: salt.to_string(),
            discord: None,
            modified_at: DateTime::from_millis(Utc::now().timestamp_millis()),
            created_at: DateTime::from_millis(Utc::now().timestamp_millis()),
        }
    }
}