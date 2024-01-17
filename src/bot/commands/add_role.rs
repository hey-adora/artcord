use crate::database::create_database::DB;
use crate::database::models::allowed_role::AllowedRole;
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
) -> Result<(), crate::bot::commands::CommandError> {
    let role_option = get_option_role(command.data.options.get(0))?;
    let feature_option = get_option_string(command.data.options.get(1))?;

    let role = db
        .allowed_role_find_one(
            &guild_id.to_string(),
            &role_option.id.to_string(),
            feature_option,
        )
        .await?;

    if let Some(role) = role {
        return Err(super::CommandError::AlreadyExists(format!(
            "Role '{}'",
            role.name
        )));
    }

    is_valid_role_feature(feature_option)?;

    let allowed_role = AllowedRole {
        _id: mongodb::bson::oid::ObjectId::new(),
        id: role_option.id.to_string(),
        guild_id: guild_id.to_string(),
        name: role_option.name.clone(),
        feature: (*feature_option).clone(),
        created_at: mongodb::bson::DateTime::now(),
        modified_at: mongodb::bson::DateTime::now(),
    };

    db.allowed_role_insert_one(allowed_role).await?;

    crate::bot::commands::show_roles::run(ctx, command, db, guild_id).await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("add_role")
        .description("Add role to whitelist of specific feature")
        .create_option(|option| {
            option
                .name("role")
                .description(format!("Role to whitelist."))
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
