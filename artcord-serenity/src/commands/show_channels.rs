use std::collections::HashMap;

use artcord_mongodb::database::DB;
use bson::doc;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{application_command::ApplicationCommandInteraction, InteractionResponseType},
    prelude::Context,
};

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
    guild_id: u64,
) -> Result<(), crate::commands::CommandError> {
    let channels = db.allowed_channel_find_all(&guild_id.to_string()).await?;

    let mut output = String::from("Features and whitelisted channels:");

    if channels.len() < 1 {
        output.push_str(" none.");
    }

    let mut unique_features: HashMap<String, String> = HashMap::new();

    for channel in channels {
        let Some(feature) = unique_features.get_mut(&channel.feature) else {
            unique_features.insert(channel.feature, format!("-<#{}>", channel.channel_id));
            continue;
        };

        feature.push_str(&format!("\n-<#{}>", channel.channel_id));
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
