use serenity::{
    builder::CreateApplicationCommand,
    model::{
        interactions::application_command::ApplicationCommandInteraction,
        prelude::{
            application_command::{CommandDataOption, CommandDataOptionValue},
            command::CommandOptionType,
        },
    },
    prelude::Context,
};

use crate::database::{AllowedChannel, DB};

use super::{get_option_channel, get_option_string, is_valid_channel_feature, CHANNEL_FEATURES};

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
    guild_id: u64,
) -> Result<(), crate::bot::commands::CommandError> {
    let channel_option = get_option_channel(command.data.options.get(0))?;
    let feature_option = get_option_string(command.data.options.get(1))?;

    is_valid_channel_feature(feature_option)?;

    let allowed_channel = AllowedChannel {
        _id: mongodb::bson::oid::ObjectId::new(),
        guild_id: guild_id.to_string(),
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

    crate::bot::commands::show_channels::run(ctx, command, db).await?;
    // Ok(format!("Channel added: {}", result.inserted_id))
    Ok(())
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
