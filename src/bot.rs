use std::collections::{HashMap, HashSet};

use cfg_if::cfg_if;

#[derive(Clone, PartialEq, Debug)]
pub enum ImgQuality {
    Low,
    Medium,
    High,
    Org,
}

impl ImgQuality {
    pub fn gen_link_preview(&self, hex: &str, format: &str) -> String {
        match self {
            ImgQuality::Low => format!("/assets/gallery/low_{}.webp", hex),
            ImgQuality::Medium => format!("/assets/gallery/medium_{}.webp", hex),
            ImgQuality::High => format!("/assets/gallery/high_{}.webp", hex),
            ImgQuality::Org => format!("/assets/gallery/org_{}.{}", hex, format),
        }
    }

    pub fn gen_link_org(hex: &str, format: &str) -> String {
        format!("/assets/gallery/org_{}.{}", hex, format)
    }

    pub fn gen_img_path_org(root: &str, hex: &str, format: &str) -> String {
        // format!("target/site/gallery/org_{}.{}", hex, format)
        format!("target/site/gallery/org_{}.{}", hex, format)
    }

    pub fn gen_img_path_high(root: &str, hex: &str) -> String {
        format!("target/site/gallery/high_{}.webp", hex)
    }

    pub fn gen_img_path_medium(root: &str, hex: &str) -> String {
        format!("target/site/gallery/medium_{}.webp", hex)
    }

    pub fn gen_img_path_low(root: &str, hex: &str) -> String {
        format!("target/site/gallery/low_{}.webp", hex)
    }

    // pub fn gen_low_medium_high_paths(hex: &str) -> [String; 4] {
    //     [
    //         ImgQuality::gen_img_path_org(hex),
    //         ImgQuality::gen_img_path_low(hex),
    //         ImgQuality::gen_img_path_medium(hex),
    //         ImgQuality::gen_img_path_high(hex),
    //     ]
    // }

    pub fn sizes() -> [u32; 3] {
        [360, 720, 1080]
    }

    // pub fn pick_quality(img: &ServerMsgImgResized) -> ImgQuality {
    //     if img.has_high {
    //         ImgQuality::High
    //     } else if img.has_medium {
    //         ImgQuality::Medium
    //     } else if img.has_low {
    //         ImgQuality::Low
    //     } else {
    //         ImgQuality::Org
    //     }
    // }
    //
    // pub fn pick_preview(img: &ServerMsgImgResized) {
    //     if img.has_high {
    //         format!("assets/gallery/high_{}.webp", img.org_hash)
    //     } else if img.has_medium {
    //         format!("assets/gallery/medium_{}.webp", img.org_hash)
    //     } else if img.has_low {
    //         format!("assets/gallery/low_{}.webp", img.org_hash)
    //     } else {
    //         format!("assets/gallery/org_{}.{}", img.org_hash, &img.format)
    //     }
    // }
}

cfg_if! {
if #[cfg(feature = "ssr")] {


    use crate::database::DB;
    use self::hooks::save_attachments::hook_save_attachments;
    use self::hooks::{
        hook_add_reaction::{hook_add_reaction},
        hook_auto_reaction::{hook_auto_react},
    };
    use futures::TryStreamExt;
    use mongodb::bson::doc;
    use serenity::client::Context;
    use serenity::framework::standard::macros::{command, group};
    use serenity::framework::standard::CommandResult;
    use serenity::framework::StandardFramework;
    use serenity::model::prelude::application_command::ApplicationCommandInteraction;
    use serenity::model::prelude::{
        ChannelId, GuildId, Interaction, InteractionResponseType, MessageId,
    };
    use serenity::model::prelude::EmojiId;
    use serenity::model::channel::Reaction;
    use serenity::model::prelude::ReactionType;
    use serenity::prelude::TypeMapKey;
    use serenity::prelude::GatewayIntents;
    use serenity::{async_trait, Client};
    use thiserror::Error;
    use tokio::sync::RwLock;
    use std::sync::Arc;
    use crate::database::AutoReaction;

    mod commands;
    mod hooks;

    use commands::FEATURE_COMMANDER;

    pub struct ArcStr;
    impl TypeMapKey for ArcStr {
        type Value = Arc<str>;
    }

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
            name => Err(crate::bot::commands::CommandError::NotImplemented(
                name.to_string(),
            )),
        }?;

        Ok(())
    }

    #[async_trait]
    impl serenity::client::EventHandler for BotHandler {

        async fn reaction_remove(&self, ctx: Context, remove_reaction: Reaction) {
            let Some(guild_id) = remove_reaction.guild_id else {
                return;
            };

            let (db, reaction_queue) = {
                let data_read = ctx.data.read().await;

                let db = data_read
                    .get::<crate::database::DB>()
                    .expect("Expected crate::database::DB in TypeMap")
                    .clone();
                let reaction_queue = data_read
                    .get::<ReactionQueue>()
                    .expect("Expected crate::database::DB in TypeMap")
                    .clone();
                (db, reaction_queue)
            };

            let allowed_guild = db.allowed_guild_exists(guild_id.0.to_string().as_str()).await;
            let Ok(allowed_guild) = allowed_guild else {
                println!("Mongodb error: {}", allowed_guild.err().unwrap());
                return;
            };
            if !allowed_guild {
                return;
            }

            let result = hook_add_reaction(&ctx, true, guild_id.0, &remove_reaction, &db).await;

            if let Err(err) = result {
                println!("{:?}", err);
                return;
            }
            // println!("removed emoji");
        }

        async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {

            let Some(guild_id) = add_reaction.guild_id else {
                return;
            };

            let db = {
                let data_read = ctx.data.read().await;

                data_read
                    .get::<crate::database::DB>()
                    .expect("Expected crate::database::DB in TypeMap")
                    .clone()
            };

            let allowed_guild = db.allowed_guild_exists(guild_id.0.to_string().as_str()).await;
            let Ok(allowed_guild) = allowed_guild else {
                println!("Mongodb error: {}", allowed_guild.err().unwrap());
                return;
            };
            if !allowed_guild {
                return;
            }


            let result = hook_add_reaction(&ctx, false, guild_id.0, &add_reaction, &db).await;

            if let Err(err) = result {
                println!("{:?}", err);
                return;
            }
            // println!("emoji_added");

            // let db = {
            //     let data_read = ctx.data.read().await;
            //
            //     data_read
            //         .get::<crate::database::DB>()
            //         .expect("Expected crate::database::DB in TypeMap")
            //         .clone()
            // };
        }

        async fn message(&self, ctx: Context, msg: serenity::model::channel::Message) {
            let Some(guild_id) = msg.guild_id else {
                return;
            };

            let time = msg.timestamp.timestamp_millis();

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

            let result = hook_save_attachments(
                &*gallery_root_dir,
                &msg.attachments,
                &db,
                time,
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

            let a = msg.react(&ctx.http, ReactionType::Custom { animated: false, id: EmojiId(1175429915999490152), name: Some(String::from("done")) }).await;

            let result = hook_auto_react(&ctx, guild_id.0, &msg, &db, false).await;

            if let Err(err) = result {
                println!("{:?}", err);
                return;
            }
        }

        async fn message_delete(
            &self,
            ctx: Context,
            _channel_id: ChannelId,
            deleted_message_id: MessageId,
            guild_id: Option<GuildId>,
        ) {
            let Some(guild_id) = guild_id else {
                return;
            };


            let db = {
                let data_read = ctx.data.read().await;

                data_read
                    .get::<crate::database::DB>()
                    .expect("Expected crate::database::DB in TypeMap")
                    .clone()
            };

                let allowed_guild = db.allowed_guild_exists(guild_id.0.to_string().as_str()).await;
                let Ok(allowed_guild) = allowed_guild else {
                    println!("Mongodb error: {}", allowed_guild.err().unwrap());
                    return;
                };
                if !allowed_guild {
                    return;
                }

            let result = db.img_hide(guild_id.0, deleted_message_id.0).await;

            let Ok(result) = result else {
                println!(
                    "ERROR: failed to hide img '{}': {}",
                    deleted_message_id.0,
                    result.err().unwrap()
                );
                return;
            };

            if result {
                println!("IMG HIDDEN: {}", deleted_message_id);
            }
        }

        async fn message_delete_bulk(
            &self,
            ctx: Context,
            _channel_id: ChannelId,
            multiple_deleted_messages_id: Vec<MessageId>,
            guild_id: Option<GuildId>,
        ) {
            let Some(guild_id) = guild_id else {
                return;
            };

            let db = {
                let data_read = ctx.data.read().await;

                data_read
                    .get::<crate::database::DB>()
                    .expect("Expected crate::database::DB in TypeMap")
                    .clone()
            };

            let allowed_guild = db.allowed_guild_exists(guild_id.0.to_string().as_str()).await;
            let Ok(allowed_guild) = allowed_guild else {
                println!("Mongodb error: {}", allowed_guild.err().unwrap());
                return;
            };
            if !allowed_guild {
                return;
            }

            for deleted_message_id in multiple_deleted_messages_id {
                let result = db.img_hide(guild_id.0, deleted_message_id.0).await;
                // let result = db
                //     .collection_img
                //     .update_one(
                //         doc! { "id": deleted_message_id.0.to_string() },
                //         doc! { "$set": { "show": false } },
                //         None,
                //     )
                //     .await;
                let Ok(_) = result else {
                    println!(
                        "ERROR: failed to hide img '{}': {}",
                        deleted_message_id.0,
                        result.err().unwrap()
                    );
                    return;
                };

                println!("IMG HIDDEN: {}", deleted_message_id);
            }
        }

        async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
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

        async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
            println!("{} is connected!", ready.user.name);

            let db = {
                let data_read = ctx.data.read().await;
                data_read
                    .get::<crate::database::DB>()
                    .expect("Expected crate::database::DB in TypeMap")
                    .clone()
            };

            for guild in ctx.cache.guilds() {
                if !db.allowed_guild_exists(guild.0.to_string().as_str()).await.expect("Failed to read database.") {
                    println!("Skipped command update for guild: {}", guild.0);
                    continue;
                }

                let _commands = GuildId::set_application_commands(&guild, &ctx.http, |commands| {
                    commands
                        .create_application_command(|command| commands::who::register(command))
                        .create_application_command(|command| commands::test::register(command))
                        .create_application_command(|command| commands::guilds::register(command))
                        .create_application_command(|command| commands::leave::register(command))
                        .create_application_command(|command| commands::sync::register(command))
                        .create_application_command(|command| commands::add_channel::register(command))
                        .create_application_command(|command| commands::add_role::register(command))
                        .create_application_command(|command| commands::remove_guild::register(command))
                        .create_application_command(|command| commands::remove_auto_emoji::register(command))
                        .create_application_command(|command| commands::reset_time::register(command))
                        .create_application_command(|command| commands::add_guild::register(command))
                        .create_application_command(|command| commands::show_guilds::register(command))
                        .create_application_command(|command| commands::remove_role::register(command))
                        .create_application_command(|command| commands::add_auto_emoji::register(command))
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

    pub struct ReactionQueue {
         pub msg_id: u64,
         pub channel_id: u64,
         pub reactions: Vec<AutoReaction>,
         pub add: bool
    }

    impl ReactionQueue {
        pub fn new(channel_id: u64, msg_id: u64, add: bool) -> Self {
            Self {
                msg_id,
                channel_id,
                reactions: Vec::new(),
                add
            }
        }
    }

    impl TypeMapKey for ReactionQueue {
        type Value = Arc<RwLock<HashMap<u64, Self>>>;
    }

    pub async fn create_bot(db: Arc<crate::database::DB>, token: String, gallery_root_dir: &str) -> serenity::Client {
        let framework = StandardFramework::new()
            .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
            .group(&GENERAL_GROUP);

        // Login with a bot token from the environment
        //let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::GUILD_MESSAGE_REACTIONS
            | GatewayIntents::MESSAGE_CONTENT;
        let client = Client::builder(token, intents)
            .event_handler(BotHandler)
            .framework(framework)
            .await
            .expect("Error creating client");

        // let allowed_roles = Arc::new(RwLock::new(HashMap::<String, AllowedRole>::new()));
        // let allowed_channels = Arc::new(RwLock::new(HashMap::<String, AllowedChannel>::new()));
        let reaction_queue = Arc::new(RwLock::new(HashMap::new()));
        {
            let mut data = client.data.write().await;
            data.insert::<crate::database::DB>(db);
            data.insert::<ReactionQueue>(reaction_queue);
            data.insert::<ArcStr>(Arc::from(gallery_root_dir));
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
 }
}
