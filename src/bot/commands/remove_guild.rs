use bson::doc;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType,
    },
    prelude::Context,
};

use crate::database::DB;

use super::{get_option_role, get_option_string, is_valid_role_feature, ROLE_FEATURES};

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
) -> Result<(), crate::bot::commands::CommandError> {
    let guild_option = get_option_string(command.data.options.get(0))?;
    let deleted = db.allowed_guild_remove_one(guild_option.as_str()).await?;

    if deleted {
        return Err(crate::bot::commands::CommandError::NotFound(format!(
            "guild: {}",
            guild_option
        )));
    }

    crate::bot::commands::show_guilds::run(ctx, command, db).await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("remove_guild")
        .description("Remove guild from whitelist")
        .create_option(|option| {
            option
                .name("role")
                .description(format!("Guild to remove."))
                .kind(CommandOptionType::Role)
                .required(true)
        })
}
