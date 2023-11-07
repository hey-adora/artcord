use bson::doc;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        interactions::application_command::ApplicationCommandInteraction,
        prelude::{
            application_command::{CommandDataOption, CommandDataOptionValue},
            command::CommandOptionType,
            InteractionResponseType,
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

    let result = db
        .collection_allowed_channel
        .delete_one(
            doc! { "id": channel_option.id.0.to_string(), "feature": feature_option },
            None,
        )
        .await?;

    if result.deleted_count < 1 {
        return Err(crate::bot::commands::CommandError::NotFound(format!(
            "feature: {} in <#{}>",
            feature_option, channel_option.id
        )));
    }

    let content = format!(
        "feature: {} in <#{}> was removed.",
        feature_option, channel_option.id
    );

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("remove_channel")
        .description("Remove channel from whitelist of specific feature")
        .create_option(|option| {
            option
                .name("channel")
                .description(format!("Channel to remove."))
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
