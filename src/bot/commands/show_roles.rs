use std::collections::HashMap;

use bson::doc;
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
    guild_id: u64,
) -> Result<(), crate::bot::commands::CommandError> {
    let roles = db
        .collection_allowed_role
        .find(doc! { "guild_id": guild_id.to_string() }, None)
        .await?;
    let roles = roles.try_collect().await.unwrap_or_else(|_| vec![]);

    let mut output = String::from("Features and whitelisted roles:");

    if roles.len() < 1 {
        output.push_str(" none.");
    }

    let mut unique_features: HashMap<String, String> = HashMap::new();

    for role in roles {
        let Some(mut feature) = unique_features.get_mut(&role.feature) else {
            unique_features.insert(role.feature, format!("-{}", role.name));
            continue;
        };

        feature.push_str(&format!("\n-{}", role.name));
    }

    for (feature, roles) in unique_features {
        output.push_str(&format!("\n{}:\n{}", feature, roles));
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
        .name("show_roles")
        .description("Show whitelisted roles for specific features.")
}
