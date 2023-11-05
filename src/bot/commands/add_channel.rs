use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{CommandDataOption, CommandDataOptionValue},
        command::CommandOptionType,
    },
};

use crate::database::{AllowedChannel, DB};

use super::{get_option_channel, get_option_string, is_valid_channel_feature, CHANNEL_FEATURES};

pub async fn run(
    options: &[CommandDataOption],
    db: &DB,
) -> Result<String, crate::bot::commands::CommandError> {
    let channel_option = get_option_channel(options.get(0))?;
    let feature_option = get_option_string(options.get(1))?;

    is_valid_channel_feature(feature_option)?;

    let allowed_channel = AllowedChannel {
        id: channel_option.id.to_string(),
        name: channel_option.name.clone().unwrap_or(String::from("none")),
        feature: (*feature_option).clone(),
        created_at: mongodb::bson::DateTime::now(),
        modified_at: mongodb::bson::DateTime::now(),
    };

    let result = db
        .collection_allowed_channel
        .insert_one(allowed_channel, None)
        .await?;

    Ok(format!("Channel added: {}", result.inserted_id))
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("add_channel")
        .description("Add channel to whitelist of specific feature")
        .create_option(|option| {
            option
                .name("channel")
                .description(format!("Channel to whitelist."))
                .kind(CommandOptionType::Channel)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("feature")
                .description(format!("Features: {:?}.", CHANNEL_FEATURES))
                .kind(CommandOptionType::String)
                .required(true)
        })
}
