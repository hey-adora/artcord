use bson::oid::ObjectId;
use bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllowedGuild {
    pub _id: ObjectId,
    pub id: String,
    pub name: String,
    pub modified_at: DateTime,
    pub created_at: DateTime,
}

impl AllowedGuild {
    pub fn new(id: String, name: String) -> Self {
        Self {
            _id: ObjectId::new(),
            id,
            name,
            created_at: DateTime::now(),
            modified_at: DateTime::now(),
        }
    }
}
