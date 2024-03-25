use artcord_mongodb::database::DB;
use bson::doc;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType,
        InteractionResponseType,
    },
    prelude::Context,
};

use super::{get_option_channel, get_option_string, is_valid_channel_feature, CHANNEL_FEATURES};

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
    _guild_id: u64,
) -> Result<(), crate::commands::CommandError> {
    let channel_option = get_option_channel(command.data.options.get(0))?;
    let feature_option = get_option_string(command.data.options.get(1))?;

    is_valid_channel_feature(feature_option)?;

    let deleted_count = db
        .allowed_channel_remove(&channel_option.id.0.to_string(), feature_option)
        .await?;

    if deleted_count < 1 {
        return Err(crate::commands::CommandError::NotFound(format!(
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
