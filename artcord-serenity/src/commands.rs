use std::num::ParseIntError;

use artcord_state::global;
use chrono::Utc;
use serenity::model::{
    channel::ReactionType,
    id::EmojiId,
    prelude::{
        application_command::{CommandDataOption, CommandDataOptionValue},
        channel::PartialChannel,
        guild::Role,
    },
};
use thiserror::Error;

use super::hooks::save_attachments::SaveAttachmentsError;

pub mod add_auto_emoji;
pub mod add_channel;
pub mod add_guild;
pub mod add_role;
pub mod guilds;
pub mod leave;
pub mod reset_time;
// pub mod add_reaction_channel;
pub mod remove_auto_emoji;
pub mod remove_channel;
pub mod remove_guild;
pub mod remove_role;
pub mod show_channels;
pub mod show_guilds;
pub mod show_roles;
pub mod sync;
pub mod test;
pub mod verify;
pub mod who;

pub const FEATURE_GALLERY: &str = "gallery";
pub const FEATURE_COMMANDER: &str = "commander";
pub const FEATURE_REACT: &str = "react";
// pub const REACT_FEATURES: [&str; 1] = [FEATURE_GALLERY];
pub const CHANNEL_FEATURES: [&str; 2] = [FEATURE_GALLERY, FEATURE_REACT];
pub const ROLE_FEATURES: [&str; 2] = [FEATURE_COMMANDER, FEATURE_GALLERY];

// pub fn is_valid_react_feature(feature: &str) -> Result<(), CommandError> {
//     for feat in REACT_FEATURES {
//         if feature == feat.to_string().as_str() {
//             return Ok(());
//         }
//     }
//     return Err(CommandError::OptionNotFound(format!(
//         "feature '{}' not found in {:?}",
//         feature, &REACT_FEATURES
//     )));
// }

pub fn is_valid_channel_feature(feature: &str) -> Result<(), CommandError> {
    for feat in CHANNEL_FEATURES {
        if feature == feat.to_string().as_str() {
            return Ok(());
        }
    }
    return Err(CommandError::OptionNotFound(format!(
        "feature '{}' not found in {:?}",
        feature, &CHANNEL_FEATURES
    )));
}

pub fn is_valid_role_feature(feature: &str) -> Result<(), CommandError> {
    for feat in ROLE_FEATURES {
        if feature == feat.to_string().as_str() {
            return Ok(());
        }
    }
    return Err(CommandError::OptionNotFound(format!(
        "feature '{}' not found in {:?}",
        feature, &ROLE_FEATURES
    )));
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("ToReactionTypeError: {0}")]
    ToReactionTypeError(#[from] ToReactionTypeError),

    #[error("Failed to parse number: {0}")]
    Number(#[from] ParseIntError),

    #[error("Failed to get time")]
    Time,

    #[error("Option not found: {0}")]
    OptionNotFound(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Mongodb error: {0}")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Serenity error: {0}")]
    Serenity(#[from] serenity::Error),

    #[error("Failed to save attachments: {0}")]
    Attachments(#[from] SaveAttachmentsError),

    // #[error("Mongodb collect error: {0}")]
    // Mongo(#[from] mongodb::error::Error),
    #[error("Command not implemented: {0}")]
    NotImplemented(String),

    #[error("Failed to make http request {0}.")]
    Request(#[from] reqwest::Error),
}

macro_rules! get_option {
    ($kind:ident, $rt:ty, $name: ident, $err: expr) => {
        pub fn $name(option: Option<&CommandDataOption>) -> Result<&$rt, CommandError> {
            let Some(option) = option else {
                return Err(CommandError::OptionNotFound($err));
            };

            let Some(option) = option.resolved.as_ref() else {
                return Err(CommandError::OptionNotFound($err));
            };

            let CommandDataOptionValue::$kind(channel_option) = option else {
                return Err(CommandError::OptionNotFound($err));
            };

            Ok(channel_option)
        }
    };
}

get_option!(
    Channel,
    PartialChannel,
    get_option_channel,
    String::from("Channel option was not provided.")
);

get_option!(
    String,
    String,
    get_option_string,
    String::from("String option was not provided.")
);

get_option!(
    Integer,
    i64,
    get_option_integer,
    String::from("Integer option was not provided.")
);

get_option!(
    Role,
    Role,
    get_option_role,
    String::from("Role option was not provided.")
);

pub fn to_reaction_type(auto_reaction: global::DbAutoReaction) -> Result<ReactionType, ToReactionTypeError> {
    let reaction: ReactionType = if let Some(unicode) = auto_reaction.unicode {
        ReactionType::Unicode(unicode)
    } else {
        let id = auto_reaction
            .emoji_id
            .ok_or(ToReactionTypeError::Id(format!("{}", "FIX LATER")))?
            .parse::<u64>()?;
        let name = auto_reaction
            .name
            .ok_or(ToReactionTypeError::Name(format!("{}", "FIX LATER")))?;

        ReactionType::Custom {
            animated: auto_reaction.animated,
            id: EmojiId(id),
            name: Some(name),
        }
    };

    Ok(reaction)
}

pub fn from_reaction_type(
    guild_id: u64,
    reaction_type: ReactionType,
) -> Result<global::DbAutoReaction, FromReactionTypeError> {
    let auto_reaction = match reaction_type {
        serenity::model::prelude::ReactionType::Unicode(s) => {
            let auto_reaction = global::DbAutoReaction::new(guild_id.to_string(), Some(s), None, None, false);

            Ok(auto_reaction)
        }
        serenity::model::prelude::ReactionType::Custom { animated, id, name } => {
            let auto_reaction = global::DbAutoReaction::new(
                guild_id.to_string(),
                None,
                Some(id.0.to_string()),
                name,
                animated,
            );

            Ok(auto_reaction)
        }
        _ => Err(FromReactionTypeError::Invalid),
    }?;
    Ok(auto_reaction)
}

pub fn from_reaction_type_vec(
    guild_id: u64,
    reaction_types: Vec<ReactionType>,
) -> Result<Vec<global::DbAutoReaction>, FromReactionTypeError> {
    let mut auto_reactions: Vec<global::DbAutoReaction> = Vec::new();
    for reaction in reaction_types {
        let auto_reaction = match reaction {
            serenity::model::prelude::ReactionType::Unicode(s) => {
                let auto_reaction =
                global::DbAutoReaction::new(guild_id.to_string(), Some(s), None, None, false);

                Ok(auto_reaction)
            }
            serenity::model::prelude::ReactionType::Custom { animated, id, name } => {
                let auto_reaction = global::DbAutoReaction::new(
                    guild_id.to_string(),
                    None,
                    Some(id.0.to_string()),
                    name,
                    animated,
                );

                Ok(auto_reaction)
            }
            _ => Err(FromReactionTypeError::Invalid),
        }?;
        auto_reactions.push(auto_reaction);
    }

    Ok(auto_reactions)
}

pub fn to_reaction_type_vec(
    auto_reactions: Vec<global::DbAutoReaction>,
) -> Result<Vec<ReactionType>, ToReactionTypeError> {
    let mut output: Vec<ReactionType> = Vec::with_capacity(auto_reactions.len());
    for reaction in auto_reactions {
        output.push(to_reaction_type(reaction)?);
    }
    Ok(output)
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
