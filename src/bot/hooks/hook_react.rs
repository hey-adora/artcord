use bson::doc;
use std::path::Path;
use thiserror::Error;

use serenity::{
    model::{channel::Message, prelude::Emoji},
    prelude::Context,
};

use crate::database::DB;

use super::save_attachments::SaveAttachmentsError;

pub async fn hook_react(
    ctx: Context,
    guild_id: u64,
    msg: Message,
    db: &DB,
) -> Result<(), HookReactError> {
    let mut should_react = false;
    if msg.attachments.len() > 0 || msg.embeds.len() > 0 {
        // let role = db
        // .collection_allowed_role
        // .find_one(
        //     doc! { "guild_id": guild_id.to_string(), "id": role_option.id.to_string(), "feature": feature_option },
        //     None,
        // )
        // .await?;

        for attachment in msg.attachments {
            let Some(content_type) = attachment.content_type.as_ref() else {
                continue;
            };

            let Ok(_) = (match content_type.as_str() {
                "image/png" => Ok("png"),
                "image/jpeg" => Ok("jpeg"),
                _ => Err(()),
            }) else {
                continue;
            };

            should_react = true;
            break;
        }

        if (!should_react) {
            for embed in msg.embeds {
                let Some(img) = embed.image else {
                    continue;
                };

                let Some(extension) = Path::new(&img.url).extension() else {
                    continue;
                };

                let Some(extension) = extension.to_str() else {
                    continue;
                };

                if !["png", "jpg", "jpeg", "webp"].contains(&extension.to_lowercase().as_str()) {
                    continue;
                }

                should_react = true;
                break;
            }
        }

        // if (should_react) {
        //     let emoji = ctx.http.get_emoji(guild_id, emoji_id).await?;
        //     msg.react(&ctx.http, );
        // }
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum HookReactError {}
