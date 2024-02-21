use bson::doc;
use futures::TryStreamExt;
use serenity::client::Context;
use serenity::model::prelude::{Interaction, InteractionResponseType};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use crate::bot::{commands};
use crate::bot::commands::FEATURE_COMMANDER;
use crate::database::DB;
use thiserror::Error;
use crate::bot::create_bot::ArcStr;

pub async fn interaction_create(ctx: Context, interaction: Interaction) {
    if let Interaction::ApplicationCommand(command) = interaction {
        let Some(guild_id) = command.guild_id else {
            return;
        };
        let (db, gallery_root_dir) = {
            let data_read = ctx.data.read().await;

            let db = data_read
                .get::<crate::database::DB>()
                .expect("Expected crate::database::DB in TypeMap")
                .clone();
            let gallery_root_dir = data_read
                .get::<ArcStr>()
                .expect("Expected crate::database::DB in TypeMap")
                .clone();
            (db, gallery_root_dir)
        };
        let allowed_guild = db.allowed_guild_exists(guild_id.0.to_string().as_str()).await;
        let Ok(allowed_guild) = allowed_guild else {
            println!("Mongodb error: {}", allowed_guild.err().unwrap());
            return;
        };
        if !allowed_guild {
            return;
        }
        let result = resolve_command(&gallery_root_dir, &ctx, &command, &db).await;
        if let Err(err) = result {
            println!("Error: {}", err);
            // command.
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(err.to_string()))
                })
                .await
            {
                let err_str = why.to_string();
                if err_str == "Interaction has already been acknowledged." {
                    if let Err(why) = command
                        .edit_original_interaction_response(&ctx.http, |message| {
                            message.content(format!("Error: {}", err))
                        })
                        .await
                    {
                        println!("Cannot respond to slash command: {}", why);
                    }
                }
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

pub async fn resolve_command(
    gallery_root_dir: &str,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB
) -> Result<(), ResolveCommandError> {
    let command_name = command.data.name.as_str();
    let guild_id = command
        .guild_id
        .as_ref()
        .ok_or(ResolveCommandError::MustRunInGuild)?;
    let member = command
        .member
        .as_ref()
        .ok_or(ResolveCommandError::MustRunInGuild)?;

    let roles = db
        .collection_allowed_role
        .find(doc! { "guild_id": guild_id.to_string() }, None)
        .await?
        .try_collect()
        .await
        .unwrap_or_else(|_| vec![]);

    let no_roles_set = roles.len() < 1;
    let user_commander_authorized = roles
        .iter()
        .filter(|r| r.feature == FEATURE_COMMANDER)
        .position(|r| {
            member
                .roles
                .iter()
                .position(|m| m.0.to_string() == r.id)
                .is_some()
        })
        .is_some();

    let user_gallery_authorized = roles
        .iter()
        .filter(|r| r.feature == FEATURE_COMMANDER)
        .position(|r| {
            member
                .roles
                .iter()
                .position(|m| m.0.to_string() == r.id)
                .is_some()
        })
        .is_some();

    if !no_roles_set && !user_commander_authorized && !user_gallery_authorized {
        return Err(ResolveCommandError::Unauthorized);
    }

    match command_name {
        "add_role" if user_commander_authorized || no_roles_set => {
            commands::add_role::run(&ctx, &command, &db, guild_id.0).await
        }
        "reset_time" if user_commander_authorized || no_roles_set => {
            commands::reset_time::run(&ctx, &command, &db, guild_id.0).await
        }
        "add_channel" if user_commander_authorized || no_roles_set => {
            commands::add_channel::run(&ctx, &command, &db, guild_id.0).await
        }
        "remove_channel" if user_commander_authorized || no_roles_set => {
            commands::remove_channel::run(&ctx, &command, &db, guild_id.0).await
        }
        "remove_role" if user_commander_authorized || no_roles_set => {
            commands::remove_role::run(&ctx, &command, &db, guild_id.0).await
        }
        "show_channels" if user_commander_authorized || no_roles_set => {
            commands::show_channels::run(&ctx, &command, &db, guild_id.0).await
        }
        "show_roles" if user_commander_authorized || no_roles_set => {
            commands::show_roles::run(&ctx, &command, &db, guild_id.0).await
        }
        "add_guild" if user_commander_authorized || no_roles_set => {
            commands::add_guild::run(&ctx, &command, &db).await
        }
        "remove_guild" if user_commander_authorized || no_roles_set => {
            commands::remove_guild::run(&ctx, &command, &db).await
        }
        "show_guilds" if user_commander_authorized || no_roles_set => {
            commands::show_guilds::run(&ctx, &command, &db).await
        }
        "joined_guilds" if user_commander_authorized || no_roles_set => {
            commands::guilds::run(&ctx, &command, &db, guild_id.0).await
        }
        "leave" if user_commander_authorized || no_roles_set => {
            commands::leave::run(&ctx, &command, &db, guild_id.0).await
        }
        "add_auto_emoji" if user_commander_authorized || no_roles_set => {
            commands::add_auto_emoji::run(&ctx, &command, &db, guild_id.0).await
        }
        "remove_auto_emoji" if user_commander_authorized || no_roles_set => {
            commands::remove_auto_emoji::run(&ctx, &command, &db, guild_id.0).await
        }
        "sync" if user_gallery_authorized || no_roles_set => {
            commands::sync::run(gallery_root_dir, &ctx, &command, &db, guild_id.0).await
        }
        "verify" if user_gallery_authorized || no_roles_set => {
            commands::verify::run(&ctx, &command, &db).await
        }
        name => Err(crate::bot::commands::CommandError::NotImplemented(
            name.to_string(),
        )),
    }?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum ResolveCommandError {
    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Command error: {0}.")]
    Command(#[from] crate::bot::commands::CommandError),

    #[error("Not authorized.")]
    Unauthorized,

    #[error("Command must be run in guild.")]
    MustRunInGuild,
}