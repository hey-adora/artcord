use bson::doc;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType,
    },
    prelude::Context,
};

use crate::database::{AllowedGuild, AllowedRole, DB};

use super::{get_option_role, get_option_string, is_valid_role_feature, ROLE_FEATURES};

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
) -> Result<(), crate::bot::commands::CommandError> {
    let guild_option = get_option_string(command.data.options.get(0))?;
    let guild = ctx.http.get_guild(guild_option.parse::<u64>()?).await?;

    let allowed_guild = AllowedGuild {
        _id: mongodb::bson::oid::ObjectId::new(),
        id: guild_option.to_owned(),
        name: guild.name,
        created_at: mongodb::bson::DateTime::now(),
        modified_at: mongodb::bson::DateTime::now(),
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
