use crate::database::create_database::DB;
use crate::database::models::allowed_guild::AllowedGuild;
use bson::doc;
use chrono::Utc;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType,
    },
    prelude::Context,
};

use super::{get_option_role, get_option_string, is_valid_role_feature, ROLE_FEATURES};

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
) -> Result<(), crate::bot::commands::CommandError> {
    let guild_option = get_option_string(command.data.options.get(0))?;
    let guild = ctx.http.get_guild(guild_option.parse::<u64>()?).await?;

    let allowed_guild = AllowedGuild {
        guild_id: guild_option.to_owned(),
        name: guild.name,
        created_at: Utc::now().timestamp_millis(),
        modified_at: Utc::now().timestamp_millis(),
    };

    let result = db.allowed_guild_insert(allowed_guild).await?;
    if result.is_some() {
        return Err(super::CommandError::AlreadyExists(format!(
            "Guild '{}'",
            guild.id
        )));
    }

    crate::bot::commands::show_guilds::run(ctx, command, db).await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("add_guild")
        .description("Whitelist guilds that bot will work on.")
        .create_option(|option| {
            option
                .name("guild")
                .description(format!("Guild to whitelist."))
                .kind(CommandOptionType::String)
                .required(true)
        })
}
