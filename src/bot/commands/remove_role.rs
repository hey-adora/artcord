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
    guild_id: u64,
) -> Result<(), crate::bot::commands::CommandError> {
    let role_option = get_option_role(command.data.options.get(0))?;
    let feature_option = get_option_string(command.data.options.get(1))?;

    is_valid_role_feature(feature_option)?;

    let result = db
        .collection_allowed_role
        .delete_one(
            doc! { "guild_id": guild_id.to_string(), "id": role_option.id.0.to_string(), "feature": feature_option },
            None,
        )
        .await?;

    if result.deleted_count < 1 {
        return Err(crate::bot::commands::CommandError::NotFound(format!(
            "feature: {} in {}",
            feature_option, role_option.name
        )));
    }

    let _content = format!(
        "feature: {} in {} was removed.",
        feature_option, role_option.name
    );

    crate::bot::commands::show_roles::run(ctx, command, db, guild_id).await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("remove_role")
        .description("Remove role from whitelist of specific feature")
        .create_option(|option| {
            option
                .name("role")
                .description(format!("Channel to remove."))
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
