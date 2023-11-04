use crate::database::{AllowedChannel, AllowedRole, User, DB};
use anyhow::anyhow;
use chrono::Utc;
use image::EncodableLayout;
use mongodb::bson::spec::BinarySubtype;
use mongodb::bson::{doc, Binary};
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::framework::StandardFramework;
use serenity::http::CacheHttp;
use serenity::model::application::command::Command;
use serenity::model::channel::Attachment;
use serenity::model::id::GuildId;
use serenity::model::prelude::{Interaction, InteractionResponseType};
use serenity::prelude::GatewayIntents;
use serenity::{async_trait, Client};
use std::collections::HashMap;
use std::fs::File;
use std::future::Future;
use std::hash::Hash;
use std::io::{Cursor, Write};
use std::num::ParseIntError;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LockResult};
use std::{env, fs, io};
use thiserror::Error;
use tokio::sync::RwLock;
use webp::WebPEncodingError;

use self::hooks::save_attachments::hook_save_attachments;

mod commands;
mod events;
mod hooks;

#[group]
#[commands(ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &serenity::model::channel::Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

struct BotHandler;

#[async_trait]
impl serenity::client::EventHandler for BotHandler {
    async fn message(&self, ctx: Context, msg: serenity::model::channel::Message) {
        let (db) = {
            let data_read = ctx.data.read().await;

            data_read
                .get::<crate::database::DB>()
                .expect("Expected crate::database::DB in TypeMap")
                .clone()
        };

        let result = hook_save_attachments(
            &msg.attachments,
            &db,
            msg.channel_id.0,
            msg.id.0,
            msg.author.id.0,
            msg.author.name.clone(),
            msg.author.avatar.clone(),
        )
        .await;

        if let Err(err) = result {
            println!("{:?}", err);
            return;
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            //println!("Received command interaction: {:#?}", command);
            let db = {
                let data_read = ctx.data.read().await;
                data_read
                    .get::<crate::database::DB>()
                    .expect("Expected crate::database::DB in TypeMap")
                    .clone()
            };
            //
            // let content = match command.data.name.as_str() {
            //     "test" => {
            //         // let db = {
            //         //     let data_read = ctx.data.read().await;
            //         //     data_read.get::<crate::database::DB>().expect("Expected crate::database::DB in TypeMap").clone()
            //         // };
            //         //
            //         // let img = crate::database::Img::default();
            //         // let r = db.collection_img.insert_one(&img, None).await;
            //         // let msg = match r {
            //         //     Ok(r) => format!("IMG Inserted: {}", r.inserted_id),
            //         //     Err(e) => format!("Failed to insert IMG: {}", e)
            //         // };
            //
            //         let msg = "wow".to_string();
            //
            //         msg
            //     }
            //     "who" => "WONDERINOOOOOOOOO".to_string(),
            //     "sync" => commands::sync::run(&command.data.options, &db).await,
            //     "add_channel" => commands::add_channel::run(&command.data.options, &db),
            //     _ => "not implemented >:3".to_string(),
            // };
            let content: Result<String, crate::bot::commands::CommandError> = match command
                .data
                .name
                .as_str()
            {
                "add_channel" => commands::add_channel::run(&command.data.options, &db).await,
                "remove_channel" => commands::remove_channel::run(&command.data.options, &db).await,
                "show_channels" => commands::show_channels::run(&command.data.options, &db).await,
                name => Err(crate::bot::commands::CommandError::NotImplemented(
                    name.to_string(),
                )),
            };

            let content = match content {
                Ok(str) => str,
                Err(err) => format!("Error: {:?}.", err),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        println!("{} is connected!", ready.user.name);

        for guild in ctx.cache.guilds() {
            let commands = GuildId::set_application_commands(&guild, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| commands::who::register(command))
                    .create_application_command(|command| commands::test::register(command))
                    .create_application_command(|command| commands::sync::register(command))
                    .create_application_command(|command| commands::add_channel::register(command))
                    .create_application_command(|command| {
                        commands::remove_channel::register(command)
                    })
                    .create_application_command(|command| {
                        commands::show_channels::register(command)
                    })
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
