// #[cfg(feature = "ssr")]
// pub mod serd;

//use crate::bot::img_quality::ImgQuality;
// use bson::oid::ObjectId;
// use bson::DateTime;
use field_types::FieldName;
use serde::{Deserialize, Serialize};
// use std::num::ParseIntError;
//use thiserror::Error;

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, FieldName)]
pub struct AutoReaction {
    pub emoji_id: Option<String>,
    pub guild_id: String,
    pub unicode: Option<String>,
    pub name: Option<String>,
    pub animated: bool,
    pub modified_at: i64,
    pub created_at: i64,
}

// #[derive(Error, Debug)]
// pub enum FromReactionTypeError {
//     #[error("Invalid ReactionType")]
//     Invalid,
// }

// #[derive(Error, Debug)]
// pub enum ToReactionTypeError {
//     #[error("Missing reaction id: {0}")]
//     Id(String),

//     #[error("Missing reaction name: {0}")]
//     Name(String),

//     #[error("Failed to parse id: {0}")]
//     ParseNumber(#[from] ParseIntError),
// }
