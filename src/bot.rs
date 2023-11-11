use self::hooks::save_attachments::hook_save_attachments;
use futures::TryStreamExt;
use mongodb::bson::doc;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::framework::StandardFramework;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{GuildId, Interaction, InteractionResponseType};
use serenity::prelude::GatewayIntents;
use serenity::{async_trait, Client};
use std::env;
use thiserror::Error;

mod commands;
mod events;
mod hooks;

use commands::FEATURE_COMMANDER;

#[group]
#[commands(ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &serenity::model::channel::Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

struct BotHandler;

pub async fn resolve_command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
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

    let db = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<crate::database::DB>()
            .expect("Expected crate::database::DB in TypeMap")
            .clone()
    };

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
        c if c == "add_role" && (user_commander_authorized || no_roles_set) => {
            commands::add_role::run(&ctx, &command, &db, guild_id.0).await
        }
        c if c == "add_channel" && (user_commander_authorized || no_roles_set) => {
            commands::add_channel::run(&ctx, &command, &db, guild_id.0).await
        }
        c if c == "remove_channel" && (user_commander_authorized || no_roles_set) => {
            commands::remove_channel::run(&ctx, &command, &db, guild_id.0).await
        }
        c if c == "remove_role" && (user_commander_authorized || no_roles_set) => {
            commands::remove_role::run(&ctx, &command, &db, guild_id.0).await
        }
        c if c == "show_channels" && (user_commander_authorized || no_roles_set) => {
            commands::show_channels::run(&ctx, &command, &db).await
        }
        c if c == "show_roles" && (user_commander_authorized || no_roles_set) => {
            commands::show_roles::run(&ctx, &command, &db, guild_id.0).await
        }
        c if c == "sync" && (user_gallery_authorized || no_roles_set) => {
            commands::sync::run(&ctx, &command, &db, guild_id.0).await
        }
        name => Err(crate::bot::commands::CommandError::NotImplemented(
            name.to_string(),
        )),
    }?;

    Ok(())
}

#[async_trait]
impl serenity::client::EventHandler for BotHandler {
    async fn message(&self, ctx: Context, msg: serenity::model::channel::Message) {
        let Some(guild_id) = msg.guild_id else {
            return;
        };

        let db = {
            let data_read = ctx.data.read().await;

            data_read
                .get::<crate::database::DB>()
                .expect("Expected crate::database::DB in TypeMap")
                .clone()
        };

        let result = hook_save_attachments(
            &msg.attachments,
            &db,
            guild_id.0,
            msg.channel_id.0,
            msg.id.0,
            msg.author.id.0,
            msg.author.name.clone(),
            msg.author.avatar.clone(),
            false,
        )
        .await;

        if let Err(err) = result {
            println!("{:?}", err);
            return;
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let result = resolve_command(&ctx, &command).await;
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

    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        println!("{} is connected!", ready.user.name);

        for guild in ctx.cache.guilds() {
            let _commands = GuildId::set_application_commands(&guild, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| commands::who::register(command))
                    .create_application_command(|command| commands::test::register(command))
                    .create_application_command(|command| commands::sync::register(command))
                    .create_application_command(|command| commands::add_channel::register(command))
                    .create_application_command(|command| commands::add_role::register(command))
                    .create_application_command(|command| commands::remove_role::register(command))
                    // .create_application_command(|command| commands::sync::register(command))
                    .create_application_command(|command| {
                        commands::remove_channel::register(command)
                    })
                    .create_application_command(|command| {
                        commands::show_channels::register(command)
                    })
                    .create_application_command(|command| commands::show_roles::register(command))
            })
            .await;
            println!("Commands updated for guild id: {}", &guild);
            // println!(
            //     "Commands updated for guild id: {}, with commands: {:#?}",
            //     &guild, commands
            // );
            // let guild_command = Command::create_global_application_command(&ctx.http, |command| {
            //     commands
            // })
        }
    }
}

pub async fn create_bot(db: crate::database::DB) -> serenity::Client {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_BOT_TOKEN").expect("token");
    //let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let client = Client::builder(token, intents)
        .event_handler(BotHandler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // let allowed_roles = Arc::new(RwLock::new(HashMap::<String, AllowedRole>::new()));
    // let allowed_channels = Arc::new(RwLock::new(HashMap::<String, AllowedChannel>::new()));

    {
        let mut data = client.data.write().await;
        data.insert::<crate::database::DB>(db);
        // data.insert::<AllowedRole>(allowed_roles);
        // data.insert::<AllowedChannel>(allowed_channels);
    }

    client
}

//
// #[derive(Error, Debug)]
// pub enum SaveUserPfpResultAsBytesError {
//     Img
//
//     #[error("Failed to save pfp: {0}")]
//     IO(#[from] std::io::Error),
// }

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
