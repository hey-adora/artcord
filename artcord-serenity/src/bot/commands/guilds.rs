use crate::database::create_database::DB;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{application_command::ApplicationCommandInteraction, InteractionResponseType},
    prelude::Context,
};

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _db: &DB,
    _guild_id: u64,
) -> Result<(), crate::bot::commands::CommandError> {
    let guilds = ctx.http.get_guilds(None, Some(100)).await?;

    let mut output = String::new();
    for guild in guilds {
        output.push_str(&format!("\n{}:{}", guild.id, guild.name));
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
        .name("joined_guilds")
        .description("Show which guilds bot is in.")
}
