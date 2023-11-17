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
    guild_id: u64,
) -> Result<(), crate::bot::commands::CommandError> {
    let guild_option = get_option_string(command.data.options.get(0))?;

    let guild = db
        .collection_allowed_guild
        .find_one(doc! { "id": guild_option }, None)
        .await?;

    if let Some(guild) = guild {
        return Err(super::CommandError::AlreadyExists(format!(
            "Guild '{}'",
            guild.id
        )));
    }

    let guild = ctx.http.get_guild(guild_option.parse::<u64>()?).await?;

    let allowed_guild = AllowedGuild {
        _id: mongodb::bson::oid::ObjectId::new(),
        id: guild_option.to_owned(),
        name: guild.name,
        created_at: mongodb::bson::DateTime::now(),
        modified_at: mongodb::bson::DateTime::now(),
    };

    let _result = db
        .collection_allowed_guild
        .insert_one(allowed_guild, None)
        .await?;

    crate::bot::commands::show_guilds::run(ctx, command, db, guild_id).await?;

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
