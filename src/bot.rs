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

mod commands;

struct ImgData {
    pub bytes: Vec<u8>,
    pub color: webp::PixelLayout,
    pub width: u32,
    pub height: u32,
}

impl ImgData {
    pub fn new(
        org_bytes: &[u8],
        img_format: image::ImageFormat,
        new_height: u32,
    ) -> Result<ImgData, ImgDataNewError> {
        //let mut img = image::io::Reader::open(file)?.decode()?;
        let mut img: image::DynamicImage = image::io::Reader::new(Cursor::new(org_bytes))
            .with_guessed_format()?
            .decode()?;
        let width = img.width();
        let height = img.height();
        if height <= new_height {
            return Err(ImgDataNewError::ImgTooSmall {
                from: height,
                to: new_height,
            });
        }
        let ratio = width as f32 / height as f32;
        let new_width = (new_height as f32 * ratio) as u32;
        img = img.resize(new_width, new_height, image::imageops::FilterType::Nearest);

        let color = ImgData::webp_color_type(img.color());

        let bytes = if color == webp::PixelLayout::Rgba {
            let rgba = img.to_rgba8();
            rgba.into_raw()
        } else {
            let rgba = img.to_rgb8();
            rgba.into_raw()
        };

        Ok(ImgData {
            bytes,
            color,
            width: new_width,
            height: new_height,
        })
    }

    pub fn encode_webp(&self) -> Result<Vec<u8>, ImgDataEncodeWebpError> {
        let webp_encoder = webp::Encoder::new(&self.bytes, self.color, self.width, self.height);
        let r = webp_encoder
            .encode_simple(false, 10f32)
            .or_else(|e| Err(ImgDataEncodeWebpError::Encode(format!("{:?}", e))))?;
        let bytes: Vec<u8> = r.to_vec();

        Ok(bytes)
    }

    fn webp_color_type(t: image::ColorType) -> webp::PixelLayout {
        match t {
            image::ColorType::Rgba8 => webp::PixelLayout::Rgba,
            image::ColorType::Rgba16 => webp::PixelLayout::Rgba,
            image::ColorType::Rgba32F => webp::PixelLayout::Rgba,
            _ => webp::PixelLayout::Rgb,
        }
    }
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

fn save_org_img(_path: &str, bytes: &[u8]) -> Result<(), SaveImgError> {
    let path = PathBuf::from(_path);

    if path.exists() {
        return Err(SaveImgError::AlreadyExist(String::from(_path)));
    }

    fs::write(&path, bytes)?;

    Ok(())
}

fn save_webp(
    _path: &str,
    bytes: &[u8],
    img_format: image::ImageFormat,
    height: u32,
) -> Result<(), SaveWebpError> {
    let path = PathBuf::from(_path);
    if !path.exists() {
        let img = ImgData::new(&bytes, img_format, height)?;
        let bytes = img.encode_webp()?;
        fs::write(&path, bytes)?;

        Ok(())
    } else {
        Err(SaveWebpError::AlreadyExist(String::from(_path)))
    }
}

async fn save_user_pfp(
    user_id: u64,
    pfp_hash: Option<String>,
    mongo_user: &Option<User>,
) -> Result<SaveUserPfpResult, SaveUserPfpError> {
    let user_pfp_hash = pfp_hash.ok_or(SaveUserPfpError::NotFound(user_id))?;

    let org_pfp_hash = match mongo_user {
        Some(user) => &user.pfp_hash,
        None => &None,
    };

    // format!("{:x}", u128::from_be_bytes(file_hash_bytes))
    //let md5_bytes: [u8; 16] = u128::from_str_radix(&user_pfp_hash, 16)?.to_be_bytes();
    let pfp_img_path = format!("assets/gallery/pfp_{}.webp", user_id);
    let pfp_url = format!(
        "https://cdn.discordapp.com/avatars/{}/{}.webp",
        user_id, user_pfp_hash
    );

    let pfp_file_exists = PathBuf::from(&pfp_img_path).exists();

    if let Some(org_pfp_hash) = org_pfp_hash {
        if *org_pfp_hash == user_pfp_hash && pfp_file_exists {
            return Ok(SaveUserPfpResult::AlreadyExists(user_pfp_hash));
        }
    }

    let pfp_img_response = reqwest::get(pfp_url).await?;
    let org_img_bytes = pfp_img_response.bytes().await?;

    if pfp_file_exists {
        fs::write(&pfp_img_path, &org_img_bytes)?;
        Ok(SaveUserPfpResult::Updated(user_pfp_hash))
    } else {
        fs::write(&pfp_img_path, &org_img_bytes)?;
        Ok(SaveUserPfpResult::Created(user_pfp_hash))
    }
}

async fn save_user(
    db: &DB,
    name: String,
    user_id: u64,
    pfp_hash: Option<String>,
) -> Result<SaveUserResult, SaveUserError> {
    let user = db
        .collection_user
        .find_one(doc! { "id": format!("{}", user_id) }, None)
        .await?;

    let pfp = save_user_pfp(user_id, pfp_hash, &user).await;
    let pfp_hash = match pfp {
        Ok(result) => Ok(Some(result.into_string())),
        Err(e) => match e {
            SaveUserPfpError::NotFound(_) => Ok(None),
            err => Err(err),
        },
    }?;

    return if let Some(user) = user {
        let mut update = doc! {};

        match pfp_hash {
            Some(bin) => match user.pfp_hash {
                Some(org_bin) => {
                    if bin != org_bin {
                        update.insert("pfp_hash", bin);
                    }
                }
                None => {
                    update.insert("pfp_hash", bin);
                }
            },
            None => match user.pfp_hash {
                Some(_) => {
                    update.insert("pfp_hash", None::<String>);
                }
                None => {}
            },
        }

        if name != user.name {
            update.insert("name", name);
        }

        if update.len() > 0 {
            update.insert("modified_at", mongodb::bson::DateTime::now());
            db.collection_img
                .update_one(
                    doc! { "id": format!("{}", user_id) },
                    doc! {
                        "$set": update.clone()
                    },
                    None,
                )
                .await?;
            Ok(SaveUserResult::Updated(format!("{}", update)))
        } else {
            Ok(SaveUserResult::None)
        }
    } else {
        let user = User {
            id: format!("{}", user_id),
            name,
            pfp_hash,
            modified_at: mongodb::bson::DateTime::now(),
            created_at: mongodb::bson::DateTime::now(),
        };

        db.collection_user.insert_one(&user, None).await?;

        Ok(SaveUserResult::Created)
    };
}

async fn save_attachment(
    db: &DB,
    user_id: u64,
    msg_id: u64,
    attachment: &Attachment,
) -> Result<SaveAttachmentResult, SaveAttachmentError> {
    let content_type = attachment
        .content_type
        .as_ref()
        .ok_or(SaveAttachmentError::ImgTypeNotFound)?;

    let format = match content_type.as_str() {
        "image/png" => Ok("png"),
        (t) => Err(SaveAttachmentError::ImgTypeUnsupported(t.to_string())),
    }?;

    let org_img_response = reqwest::get(&attachment.url).await?;
    let org_img_bytes = org_img_response.bytes().await?;

    let file_hash_bytes: [u8; 16] = hashes::md5::hash(org_img_bytes.as_bytes()).into_bytes();
    let file_hash_hex = format!("{:x}", u128::from_be_bytes(file_hash_bytes));

    let file_name = PathBuf::from(&attachment.filename);
    //let file_stem = file_name.file_stem().unwrap().to_str().unwrap();
    let file_ext = file_name.extension().unwrap().to_str().unwrap();

    let org_img_path = format!("assets/gallery/org_{}.{}", file_hash_hex, file_ext);
    let low_img_path = format!("assets/gallery/low_{}.webp", file_hash_hex);
    let medium_img_path = format!("assets/gallery/medium_{}.webp", file_hash_hex);
    let high_img_path = format!("assets/gallery/high_{}.webp", file_hash_hex);

    let mut paths = [low_img_path, medium_img_path, high_img_path];
    let mut paths_state = [false, false, false];
    let mut img_heights = [360, 720, 1080];

    let save_org_img_result = save_org_img(&org_img_path, &org_img_bytes);
    if let Err(save_org_img_result) = save_org_img_result {
        match save_org_img_result {
            SaveImgError::AlreadyExist(_) => Ok(()),
            err => Err(err),
        }?
    }

    'path_loop: for (i, path) in paths.iter().enumerate() {
        println!("{}", path);
        paths_state[i] = match save_webp(
            path,
            &org_img_bytes,
            image::ImageFormat::Png,
            img_heights[i],
        ) {
            Ok(_) => Ok(true),
            Err(e) => match e {
                SaveWebpError::AlreadyExist(p) => Ok(true),
                SaveWebpError::ImgDecoding(decoding_err) => match decoding_err {
                    ImgDataNewError::ImgTooSmall { from, to } => break 'path_loop,
                    err => Err(SaveWebpError::from(err)),
                },
                err => Err(err),
            },
        }?;
    }

    let found_img = db
        .collection_img
        .find_one(
            doc! {
                "org_hash": file_hash_hex.clone()
            },
            None,
        )
        .await?;

    return if let Some(found_img) = found_img {
        let db_img_names = ["has_low", "has_medium", "has_high"];
        let db_img_states = [found_img.has_low, found_img.has_medium, found_img.has_high];

        let mut update = doc! {};

        for (i, path_state) in paths_state.into_iter().enumerate() {
            if db_img_states[i] != path_state {
                update.insert(db_img_names[i], path_state);
            }
        }

        if update.len() > 0 {
            update.insert("modified_at", mongodb::bson::DateTime::now());
            let update_status = db
                .collection_img
                .update_one(
                    doc! { "org_hash": file_hash_hex.clone() },
                    doc! {
                        "$set": update
                    },
                    None,
                )
                .await?;
            Ok(SaveAttachmentResult::Updated(file_hash_hex))
        } else {
            Ok(SaveAttachmentResult::None(file_hash_hex))
        }
    } else {
        let mut org_img: image::DynamicImage = image::io::Reader::new(Cursor::new(&org_img_bytes))
            .with_guessed_format()?
            .decode()?;

        let img = crate::database::Img {
            user_id: format!("{}", user_id),
            msg_id: format!("{}", msg_id),
            org_hash: file_hash_hex.clone(),
            format: format!("{}", format),
            width: org_img.width(),
            height: org_img.height(),
            has_high: paths_state[2],
            has_medium: paths_state[1],
            has_low: paths_state[0],
            modified_at: mongodb::bson::DateTime::now(),
            created_at: mongodb::bson::DateTime::now(),
        };

        db.collection_img.insert_one(&img, None).await?;
        Ok(SaveAttachmentResult::Created(file_hash_hex))
    };
}

#[async_trait]
impl serenity::client::EventHandler for BotHandler {
    async fn message(&self, ctx: Context, msg: serenity::model::channel::Message) {
        let (allowed_channels, db) = {
            let data_read = ctx.data.read().await;

            (
                data_read
                    .get::<AllowedChannel>()
                    .expect("Expected AllowedChannels in TypeMap")
                    .clone(),
                data_read
                    .get::<crate::database::DB>()
                    .expect("Expected crate::database::DB in TypeMap")
                    .clone(),
            )
        };

        let allowed_channels = allowed_channels.read().await;

        if allowed_channels.len() > 0 {
            if let None = allowed_channels.get(&format!("{}_gallery", msg.channel_id.0)) {
                println!("Gallery feature cant run on channel: {}", msg.channel_id.0);
                return;
            };
        };

        println!("Running gallery feature on channel: {}", msg.channel_id.0);

        if msg.attachments.len() > 0 {
            let user = save_user(&db, msg.author.name, msg.author.id.0, msg.author.avatar).await;
            let Ok(user) = user else {
                println!("Error saving user: {}", user.err().unwrap());
                return;
            };
            println!(
                "{}",
                match user {
                    SaveUserResult::Updated(data) => format!("Updated user: {}", data),
                    SaveUserResult::Created => format!("Created user"),
                    SaveUserResult::None => format!("User is up to date"),
                }
            );

            for attachment in &msg.attachments {
                let result = save_attachment(&db, msg.author.id.0, msg.id.0, attachment).await;
                let msg = match result {
                    Ok(hash) => match hash {
                        SaveAttachmentResult::Created(hash) => {
                            format!("File '{}' saved.", hash)
                        }
                        SaveAttachmentResult::Updated(hash) => {
                            format!("File '{}' updated.", hash)
                        }
                        SaveAttachmentResult::None(hash) => {
                            format!("File '{}' already exists.", hash)
                        }
                    },
                    Err(err) => format!("Error: {}", err),
                };
                println!("{}", msg);
            }
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
            let content: Result<String, crate::bot::commands::CommandError> =
                match command.data.name.as_str() {
                    "add_channel" => commands::add_channel::run(&command.data.options, &db).await,
                    "show_channels" => {
                        commands::show_channels::run(&command.data.options, &db).await
                    }
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

    let allowed_roles = Arc::new(RwLock::new(HashMap::<String, AllowedRole>::new()));
    let allowed_channels = Arc::new(RwLock::new(HashMap::<String, AllowedChannel>::new()));

    {
        let mut data = client.data.write().await;
        data.insert::<crate::database::DB>(db);
        data.insert::<AllowedRole>(allowed_roles);
        data.insert::<AllowedChannel>(allowed_channels);
    }

    client
}

#[derive(Error, Debug)]
pub enum SaveImgError {
    #[error("Img already exists at {0}.")]
    AlreadyExist(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum SaveWebpError {
    #[error("{0}")]
    ImgDecoding(#[from] ImgDataNewError),

    #[error("{0}")]
    ImgEncoding(#[from] ImgDataEncodeWebpError),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Img already exists at {0}.")]
    AlreadyExist(String),
}

#[derive(Error, Debug)]
pub enum ImgDataNewError {
    #[error("Img invalid format: {0}.")]
    Format(#[from] std::io::Error),

    #[error("Failed to decode img: {0}.")]
    Decode(#[from] image::ImageError),

    #[error("Img too small to covert from {from:?} to {to:?}.")]
    ImgTooSmall { from: u32, to: u32 },
}

#[derive(Error, Debug)]
pub enum ImgDataEncodeWebpError {
    #[error("Webp encoding error: {0}")]
    Encode(String),
}

#[derive(Error, Debug)]
pub enum SaveAttachmentError {
    #[error("Msg content type not found.")]
    ImgTypeNotFound,

    #[error("Msg content type not found {0}.")]
    ImgTypeUnsupported(String),

    #[error("Failed downloading img {0}.")]
    Request(#[from] reqwest::Error),

    #[error("Failed to save org img {0}.")]
    ImgSave(#[from] SaveImgError),

    #[error("Failed to save webp img {0}.")]
    ImgSaveWebp(#[from] SaveWebpError),

    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Failed to decode img: {0}.")]
    Decode(#[from] image::ImageError),
}

#[derive(Error, Debug)]
pub enum SaveUserError {
    #[error("Failed to save pfp: {0}")]
    SavingPfp(#[from] SaveUserPfpError),

    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),
}

#[derive(Error, Debug)]
pub enum SaveUserPfpError {
    #[error("User '{0}' pfp not found.")]
    NotFound(u64),

    #[error("Failed downloading pfp img {0}.")]
    Request(#[from] reqwest::Error),

    #[error("Failed to convert hex to decimal {0}.")]
    HexToDec(#[from] ParseIntError),

    #[error("Failed to save pfp: {0}")]
    IO(#[from] std::io::Error),
}

pub enum SaveAttachmentResult {
    Created(String),
    Updated(String),
    None(String),
}

pub enum SaveUserPfpResult {
    AlreadyExists(String),
    Updated(String),
    Created(String),
}

impl SaveUserPfpResult {
    pub fn into_string(self) -> String {
        match self {
            SaveUserPfpResult::Created(str) => str,
            SaveUserPfpResult::Updated(str) => str,
            SaveUserPfpResult::AlreadyExists(str) => str,
        }
    }
}

pub enum SaveUserResult {
    Updated(String),
    Created,
    None,
}

//
// #[derive(Error, Debug)]
// pub enum SaveUserPfpResultAsBytesError {
//     Img
//
//     #[error("Failed to save pfp: {0}")]
//     IO(#[from] std::io::Error),
// }
