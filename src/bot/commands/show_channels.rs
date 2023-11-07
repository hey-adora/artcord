use std::collections::HashMap;

use futures::TryStreamExt;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        interactions::application_command::ApplicationCommandInteraction,
        prelude::{
            application_command::CommandDataOption, command::CommandOptionType,
            InteractionResponseType,
        },
    },
    prelude::Context,
};

use crate::database::DB;

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
) -> Result<(), crate::bot::commands::CommandError> {
    let channels = db.collection_allowed_channel.find(None, None).await?;
    let channels = channels.try_collect().await.unwrap_or_else(|_| vec![]);

    let mut output = String::from("Features and whitelisted channels:");

    if channels.len() < 1 {
        output.push_str(" none.");
    }

    let mut unique_features: HashMap<String, String> = HashMap::new();

    for channel in channels {
        let Some(mut feature) = unique_features.get_mut(&channel.feature) else {
            unique_features.insert(channel.feature, format!("-<#{}>", channel.id));
            continue;
        };

        feature.push_str(&format!("\n-<#{}>", channel.id));
    }

    for (feature, channels) in unique_features {
        output.push_str(&format!("\n{}:\n{}", feature, channels));
    }

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(output))
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("show_channels")
        .description("Show whitelisted channels for specific features.")
}
