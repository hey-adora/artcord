use std::collections::HashMap;

use crate::database::create_database::DB;
use bson::doc;
use futures::TryStreamExt;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{application_command::ApplicationCommandInteraction, InteractionResponseType},
    prelude::Context,
};

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
) -> Result<(), crate::bot::commands::CommandError> {
    let guilds = db.allowed_guild_all().await?;

    let mut output = String::from("Guilds:");

    if guilds.len() < 1 {
        output.push_str(" none.");
    }

    for guild in guilds {
        output.push_str(&format!("\n-{}:{}", guild.guild_id, guild.name));
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
        .name("show_guilds")
        .description("Show whitelisted guilds.")
}
