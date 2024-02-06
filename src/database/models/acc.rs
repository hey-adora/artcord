use crate::message::server_msg::ServerMsg;
use bson::oid::ObjectId;
use chrono::Utc;
use field_types::FieldName;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FieldName)]
pub struct Acc {
    pub id: String,
    pub email: String,
    pub password: String,
    pub verified_email: bool,
    pub email_verification_code: String,
    pub discord: Option<AccDiscord>,
    pub role: String,
    pub modified_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AccDiscord {
    pub user_id: String,

    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Role {
    Member,
    Moderator,
    Admin,
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Role::Member => "member",
                Role::Moderator => "moderator",
                Role::Admin => "admin",
            }
        )
    }
}

impl Acc {
    pub fn new(email: &str, password: &str, email_verification_code: &str) -> Acc {
        Acc {
            id: ObjectId::new().to_hex(),
            email: email.to_string(),
            verified_email: false,
            email_verification_code: email_verification_code.to_string(),
            password: password.to_string(),
            role: Role::Member.to_string(),
            discord: None,
            modified_at: Utc::now().timestamp_millis(),
            created_at: Utc::now().timestamp_millis(),
        }
    }
}
