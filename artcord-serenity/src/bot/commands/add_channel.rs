use crate::database::create_database::DB;
use crate::database::models::allowed_channel::AllowedChannel;
use chrono::Utc;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType,
    },
    prelude::Context,
};

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
        channel_id: channel_option.id.to_string(),
        guild_id: guild_id.to_string(),
        name: channel_option.name.clone().unwrap_or(String::from("none")),
        feature: (*feature_option).clone(),
        created_at: Utc::now().timestamp_millis(),
        modified_at: Utc::now().timestamp_millis(),
    };

    db.allowed_channel_insert_one(allowed_channel).await?;

    crate::bot::commands::show_channels::run(ctx, command, db, guild_id).await?;

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
