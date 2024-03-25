use artcord_mongodb::database::DB;
use bson::doc;
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
    guild_id: u64,
) -> Result<(), crate::commands::CommandError> {
    let role_option = get_option_role(command.data.options.get(0))?;
    let feature_option = get_option_string(command.data.options.get(1))?;

    is_valid_role_feature(feature_option)?;

    let deleted_count = db
        .remove_allowed_role(
            &guild_id.to_string(),
            &role_option.id.0.to_string(),
            feature_option,
        )
        .await?;

    if deleted_count < 1 {
        return Err(crate::commands::CommandError::NotFound(format!(
            "feature: {} in {}",
            feature_option, role_option.name
        )));
    }

    crate::commands::show_roles::run(ctx, command, db, guild_id).await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("remove_role")
        .description("Remove role from whitelist of specific feature")
        .create_option(|option| {
            option
                .name("role")
                .description(format!("Role to remove."))
                .kind(CommandOptionType::Role)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("feature")
                .description(format!("Features: {:?}.", ROLE_FEATURES))
                .kind(CommandOptionType::String)
                .required(true)
        })
}
