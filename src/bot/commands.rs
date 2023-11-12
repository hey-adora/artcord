use std::num::ParseIntError;

use serenity::model::prelude::{
    application_command::{CommandDataOption, CommandDataOptionValue},
    channel::PartialChannel,
    guild::Role,
};
use thiserror::Error;

use super::hooks::save_attachments::SaveAttachmentsError;

pub mod add_channel;
pub mod add_role;
pub mod guilds;
pub mod leave;
pub mod remove_channel;
pub mod remove_role;
pub mod show_channels;
pub mod show_roles;
pub mod sync;
pub mod test;
pub mod who;

pub const FEATURE_GALLERY: &str = "gallery";
pub const FEATURE_COMMANDER: &str = "commander";
pub const CHANNEL_FEATURES: [&str; 1] = [FEATURE_GALLERY];
pub const ROLE_FEATURES: [&str; 2] = [FEATURE_COMMANDER, FEATURE_GALLERY];

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
    #[error("Failed to parse number: {0}")]
    Number(#[from] ParseIntError),

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
