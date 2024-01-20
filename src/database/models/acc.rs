use crate::database::rkw::date_time::DT;
use crate::database::rkw::object_id::OBJ;
use crate::server::server_msg::ServerMsg;
use bson::oid::ObjectId;
use bson::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

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

    pub role: String,

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



#[derive(
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub enum Role {
    Member,
    Moderator,
    Admin,
}

// impl Debug for Role {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}",
//             match self {
//                 Role::Member => "member",
//                 Role::Moderator => "moderator",
//                 Role::Admin => "admin",
//             }
//         )
//     }
// }

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

// impl Into<&'static str> for Role {
//     fn into(self) -> &'static str {
//         match self {
//             Role::Member => "member",
//             Role::Moderator => "moderator",
//             Role::Admin => "admin",
//         }
//     }
// }

impl Acc {
    pub fn new(email: &str, password: &str, email_verification_code: &str) -> Acc {
        Acc {
            _id: ObjectId::new(),
            email: email.to_string(),
            verified_email: false,
            email_verification_code: email_verification_code.to_string(),
            password: password.to_string(),
            role: Role::Member.to_string(),
            //salt: salt.to_string(),
            discord: None,
            modified_at: DateTime::from_millis(Utc::now().timestamp_millis()),
            created_at: DateTime::from_millis(Utc::now().timestamp_millis()),
        }
    }
}
