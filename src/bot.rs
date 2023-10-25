use crate::database::ImgFormat;
use anyhow::anyhow;
use chrono::Utc;
use image::EncodableLayout;
use mongodb::bson::doc;
use mongodb::bson::spec::BinarySubtype;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::framework::StandardFramework;
use serenity::http::CacheHttp;
use serenity::model::application::command::Command;
use serenity::model::id::GuildId;
use serenity::model::prelude::{Interaction, InteractionResponseType};
use serenity::prelude::GatewayIntents;
use serenity::{async_trait, Client};
use std::fs::File;
use std::future::Future;
use std::hash::Hash;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

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
    ) -> anyhow::Result<ImgData> {
        //let mut img = image::io::Reader::open(file)?.decode()?;
        let mut img = image::io::Reader::new(Cursor::new(org_bytes))
            .with_guessed_format()?
            .decode()?;
        let width = img.width();
        let height = img.height();
        if height <= new_height {
            return Err(anyhow!(
                "Image is too small for resize from {} to {}",
                height,
                new_height
            ));
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

    pub fn encode_webp(&self) -> anyhow::Result<Vec<u8>> {
        let webp_encoder = webp::Encoder::new(&self.bytes, self.color, self.width, self.height);
        let r = webp_encoder
            .encode_simple(false, 10f32)
            .or_else(|e| Err(anyhow::anyhow!("{:?}", e)))?;

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

fn save_img(_path: String, bytes: &[u8]) -> Result<String, String> {
    let path = PathBuf::from(&_path);

    if path.exists() {
        return Err(format!("File already exists: {}", _path));
    }

    let io_status = fs::write(&path, bytes);
    let Ok(_) = io_status else {
        return Err(format!(
            "Failed to save {} with error: {}.",
            &_path,
            io_status.err().unwrap()
        ));
    };
    Ok(_path)
}

fn save_webp(_path: String, bytes: &[u8], img_format: image::ImageFormat, height: u32) -> bool {
    let path = PathBuf::from(&_path);
    if !path.exists() {
        let img = ImgData::new(&bytes, img_format, height);
        let Ok(img) = img else {
            println!("Error converting img: {}", img.err().unwrap());
            return false;
        };

        let bytes = img.encode_webp();
        let Ok(bytes) = bytes else {
            println!("Error converting img: {}", bytes.err().unwrap());
            return false;
        };

        let io_state = fs::write(&path, bytes);
        let Ok(_) = io_state else {
            println!(
                "Failed to save {} with error: {}.",
                _path,
                io_state.err().unwrap()
            );
            return false;
        };

        println!("Saved {}.", path.as_os_str().to_str().unwrap());

        return true;
    } else {
        println!(
            "File already exists: {}",
            path.as_os_str().to_str().unwrap()
        );
        return true;
    }
}

#[async_trait]
impl serenity::client::EventHandler for BotHandler {
    async fn message(&self, ctx: Context, msg: serenity::model::channel::Message) {
        // let guilds = ctx.cache.guilds();
        // for guild in guilds {
        //     let name = guild.name(&ctx.cache).unwrap_or_default();
        //     println!("Leaving: {}", name);
        //     guild.leave(ctx.http()).await.unwrap();
        // }
        for attachment in msg.attachments {
            let Some(content_type) = attachment.content_type else {
                println!("Failed to get content type");
                return;
            };

            //let file_name = format!("{}_{}", attachment.id, &attachment.filename);

            if content_type == "image/png" {
                println!("Downloading: {}", &attachment.filename);
            } else {
                println!("File format is {}; skipping download.", content_type);
                return;
            }

            let res = reqwest::get(attachment.url).await;
            let Ok(res) = res else {
                println!("{}", res.err().unwrap());
                return;
            };

            //let file_name = format!("assets/gallery/{}", &file_name);

            let bytes = res.bytes().await;
            let Ok(bytes) = bytes else {
                println!("Failed to get bytes: {}", bytes.err().unwrap());
                return;
            };

            //let file_hash = sha256::digest(bytes.as_bytes());
            //let mut hasher = md5::new();
            //hasher.update(bytes.as_bytes());
            //let file_hash2 = hasher.finalize();

            //let file_hash2 = md5digest(bytes.as_bytes());
            let file_hash_bytes: [u8; 16] = hashes::md5::hash(bytes.as_bytes()).into_bytes();
            let file_hash: u128 = u128::from_be_bytes(file_hash_bytes);
            let file_hash_mongo = mongodb::bson::Binary {
                subtype: BinarySubtype::Md5,
                bytes: file_hash_bytes.to_vec(),
            };

            let file_name = PathBuf::from(&attachment.filename);
            let file_stem = file_name.file_stem().unwrap().to_str().unwrap();
            let file_ext = file_name.extension().unwrap().to_str().unwrap();

            let db = {
                let data_read = ctx.data.read().await;
                data_read
                    .get::<crate::database::DB>()
                    .expect("Expected crate::database::DB in TypeMap")
                    .clone()
            };

            let found_img_status = db
                .collection_img
                .find_one(
                    doc! {
                        "org_hash": file_hash_mongo.clone()
                    },
                    None,
                )
                .await;
            let Ok(found_img) = found_img_status else {
                println!(
                    "Database error while searching for existing img '{}': {}",
                    file_hash,
                    found_img_status.err().unwrap()
                );
                return;
            };

            println!(
                "{}",
                match save_img(
                    format!("assets/gallery/org_{}.{}", file_hash, file_ext),
                    &bytes,
                ) {
                    Ok(to) => format!("Img saved to {}", to),
                    Err(e) => format!("Error: {}", e),
                }
            );

            if let Some(found_img) = found_img {
                println!("Img '{}' already exists in database.", file_hash);

                let mut update = doc! {};

                let has_low = save_webp(
                    format!("assets/gallery/low_{}.webp", file_hash),
                    &bytes,
                    image::ImageFormat::Png,
                    360,
                );

                if found_img.has_low == has_low {
                    println!("Img '{}' has_low field is correct.", file_hash);
                } else {
                    println!("Img '{}' has_low field is incorrect.", file_hash);
                    update.insert("has_low", has_low);
                }

                let has_medium = save_webp(
                    format!("assets/gallery/medium_{}.webp", file_hash),
                    &bytes,
                    image::ImageFormat::Png,
                    720,
                );

                if found_img.has_medium == has_medium {
                    println!("Img '{}' has_medium field is correct.", file_hash);
                } else {
                    println!("Img '{}' has_medium field is incorrect.", file_hash);
                    update.insert("has_medium", has_medium);
                }

                let has_high = save_webp(
                    format!("assets/gallery/high_{}.webp", file_hash),
                    &bytes,
                    image::ImageFormat::Png,
                    1080,
                );

                if found_img.has_high == has_high {
                    println!("Img '{}' has_high field is correct.", file_hash);
                } else {
                    println!("Img '{}' has_high field is incorrect.", file_hash);
                    update.insert("has_high", has_high);
                }

                if update.len() > 0 {
                    update.insert("modified_at", mongodb::bson::DateTime::now());
                    let update_status = db
                        .collection_img
                        .update_one(
                            doc! { "org_hash": file_hash_mongo },
                            doc! {
                                "$set": update
                            },
                            None,
                        )
                        .await;
                    let Ok(_) = update_status else {
                        println!(
                            "Error updating img '{}': {}",
                            file_hash,
                            update_status.err().unwrap()
                        );
                        return;
                    };
                    println!("Img '{}' updated.", file_hash);
                } else {
                    println!("Img '{}' is already up to date.", file_hash);
                }
            } else {
                //let time = Utc::now().timestamp();
                let img = crate::database::Img {
                    user_id: msg.author.id.0,
                    org_hash: file_hash_mongo,
                    format: 0,
                    has_high: save_webp(
                        format!("assets/gallery/high_{}.webp", file_hash),
                        &bytes,
                        image::ImageFormat::Png,
                        1080,
                    ),
                    has_medium: save_webp(
                        format!("assets/gallery/medium_{}.webp", file_hash),
                        &bytes,
                        image::ImageFormat::Png,
                        720,
                    ),
                    has_low: save_webp(
                        format!("assets/gallery/low_{}.webp", file_hash),
                        &bytes,
                        image::ImageFormat::Png,
                        360,
                    ),
                    modified_at: mongodb::bson::DateTime::now(),
                    created_at: mongodb::bson::DateTime::now(),
                };

                let r = db.collection_img.insert_one(&img, None).await;
                let insert_status = match r {
                    Ok(r) => format!("IMG Inserted: {}", r.inserted_id),
                    Err(e) => format!("Failed to insert IMG: {}", e),
                };

                println!("{}", insert_status);
            }

            // let Some(found_img) = found_img else {
            //     println!(
            //         "Img '{}' doesn't exist in database, it will be inserted.",
            //         file_hash
            //     );
            // };
            // println!("Img '{}' already exists in database.", file_hash);
            //
            // let a = doc! { "test": 5 };

            //let a: u8 = 0.into();
        }

        // if msg.content == "!ping" {
        //     if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
        //         println!("Error sending message: {:?}", why);
        //     }
        // }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            //println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "test" => {
                    // let db = {
                    //     let data_read = ctx.data.read().await;
                    //     data_read.get::<crate::database::DB>().expect("Expected crate::database::DB in TypeMap").clone()
                    // };
                    //
                    // let img = crate::database::Img::default();
                    // let r = db.collection_img.insert_one(&img, None).await;
                    // let msg = match r {
                    //     Ok(r) => format!("IMG Inserted: {}", r.inserted_id),
                    //     Err(e) => format!("Failed to insert IMG: {}", e)
                    // };

                    let msg = "wow".to_string();

                    msg
                }
                "who" => "WONDERINOOOOOOOOO".to_string(),
                _ => "not implemented >:3".to_string(),
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
            })
            .await;
            println!(
                "Commands updated for guild id: {}, with commands: {:#?}",
                &guild, commands
            );
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

    {
        let mut data = client.data.write().await;
        data.insert::<crate::database::DB>(db);
    }

    client
}
