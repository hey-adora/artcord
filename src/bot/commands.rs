use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{CommandDataOption, CommandDataOptionValue},
        channel::PartialChannel,
        command::CommandOptionType,
    },
};
use thiserror::Error;

pub mod add_channel;
pub mod show_channels;
pub mod sync;
pub mod test;
pub mod who;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Option not found: {0}")]
    OptionNotFound(String),

    #[error("Mongodb error: {0}")]
    Mongo(#[from] mongodb::error::Error),

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
