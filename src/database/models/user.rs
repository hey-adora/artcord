use crate::database::rkw::date_time::DT;
use crate::database::rkw::object_id::OBJ;
use bson::oid::ObjectId;
use bson::DateTime;
use serde::{Deserialize, Serialize};

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
pub struct User {
    #[with(OBJ)]
    pub _id: ObjectId,

    pub guild_id: String,
    pub id: String,
    pub name: String,
    pub pfp_hash: Option<String>,

    #[with(DT)]
    pub modified_at: DateTime,

    #[with(DT)]
    pub created_at: DateTime,
}
