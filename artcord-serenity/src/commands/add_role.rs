use artcord_mongodb::database::DB;
use artcord_state::model::allowed_role::AllowedRole;
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
    guild_id: u64,
) -> Result<(), crate::commands::CommandError> {
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
        role_id: role_option.id.to_string(),
        guild_id: guild_id.to_string(),
        name: role_option.name.clone(),
        feature: (*feature_option).clone(),
        created_at: Utc::now().timestamp_millis(),
        modified_at: Utc::now().timestamp_millis(),
    };

    db.allowed_role_insert_one(allowed_role).await?;

    crate::commands::show_roles::run(ctx, command, db, guild_id).await?;

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
