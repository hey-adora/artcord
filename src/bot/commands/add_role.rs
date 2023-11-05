use bson::doc;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{CommandDataOption, CommandDataOptionValue},
        command::CommandOptionType,
    },
};

use crate::database::{AllowedChannel, AllowedRole, DB};

use super::{
    get_option_channel, get_option_role, get_option_string, is_valid_role_feature,
    CHANNEL_FEATURES, ROLE_FEATURES,
};

pub async fn run(
    options: &[CommandDataOption],
    db: &DB,
    guild_id: u64,
) -> Result<String, crate::bot::commands::CommandError> {
    let role_option = get_option_role(options.get(0))?;
    let feature_option = get_option_string(options.get(1))?;

    let role = db
        .collection_allowed_role
        .find_one(
            doc! { "guild_id": guild_id.to_string(), "id": role_option.id.to_string() },
            None,
        )
        .await?;

    if let Some(role) = role {
        return Err(super::CommandError::AlreadyExists(format!(
            "Role '{}'",
            role.name
        )));
    }

    is_valid_role_feature(feature_option)?;

    // let _id = mongodb::bson::uuid::Uuid::new().to_string();
    // let _id = uuid::Uuid::new_v4().to_string();
    // let _id = mongodb::bson::oid::ObjectId::new().to_string();
    // println!("GENERATED ID: {}", &_id);
    let allowed_role = AllowedRole {
        _id: mongodb::bson::oid::ObjectId::new(),
        id: role_option.id.to_string(),
        guild_id: guild_id.to_string(),
        name: role_option.name.clone(),
        feature: (*feature_option).clone(),
        created_at: mongodb::bson::DateTime::now(),
        modified_at: mongodb::bson::DateTime::now(),
    };

    let result = db
        .collection_allowed_role
        .insert_one(allowed_role, None)
        .await?;

    // Ok(format!("Role added: {}", result.inserted_id))
    crate::bot::commands::show_roles::run(options, db, guild_id).await
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
