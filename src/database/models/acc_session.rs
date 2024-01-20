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
pub struct AccSession {
    #[with(OBJ)]
    pub _id: ObjectId,

    #[with(OBJ)]
    pub user_id: ObjectId,

    pub ip: String,
    pub agent: String,
    pub token: String,

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
pub struct SessionToken {
    pub token: String,
}