use crate::database::create_database::DB;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType,
        InteractionResponseType,
    },
    prelude::Context,
};

use super::get_option_string;

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _db: &DB,
    _guild_id: u64,
) -> Result<(), crate::bot::commands::CommandError> {
    let guild_id = get_option_string(command.data.options.get(0))?;
    let guild_id: u64 = guild_id.parse()?;
    ctx.http.leave_guild(guild_id).await?;

    let mut output = format!("Guild left: {}.", guild_id);

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
        .name("leave")
        .description("Leave guild.")
        .create_option(|option| {
            option
                .name("id")
                .description(format!("Guild id."))
                .kind(CommandOptionType::String)
                .required(true)
        })
}
