#[cfg(feature = "ssr")]
pub mod serd;

use crate::bot::img_quality::ImgQuality;
use bson::oid::ObjectId;
use bson::DateTime;
use serde::{Deserialize, Serialize};
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct AutoReaction {
    pub _id: ObjectId,
    pub guild_id: String,
    pub unicode: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub animated: bool,
    pub modified_at: DateTime,
    pub created_at: DateTime,
}

#[derive(Error, Debug)]
pub enum FromReactionTypeError {
    #[error("Invalid ReactionType")]
    Invalid,
}

#[derive(Error, Debug)]
pub enum ToReactionTypeError {
    #[error("Missing reaction id: {0}")]
    Id(String),

    #[error("Missing reaction name: {0}")]
    Name(String),

    #[error("Failed to parse id: {0}")]
    ParseNumber(#[from] ParseIntError),
}
