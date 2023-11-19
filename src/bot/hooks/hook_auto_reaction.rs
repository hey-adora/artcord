use bson::doc;
use rand::Rng;
use std::path::Path;
use thiserror::Error;

use serenity::{
    model::{channel::Message, prelude::Emoji},
    prelude::Context,
};

use crate::{
    bot::ReactionQueue,
    database::{AutoReaction, ToReactionTypeError, DB},
};

use super::save_attachments::SaveAttachmentsError;
use serenity::model::channel::Reaction;

pub async fn hook_auto_react(
    ctx: Context,
    guild_id: u64,
    msg: &Message,
    db: &DB,
) -> Result<(), HookReactError> {
    let mut should_react = false;

    // let reaction_queue = {
    //     let data_read = ctx.data.read().await;
    //
    //     data_read
    //         .get::<ReactionQueue>()
    //         .expect("Expected TypeMap")
    //         .clone()
    // };
    // let reaction_queue = reaction_queue.read().await;
    // if reaction_queue.get(msg.id.0).is_none() {
    //     return Ok(());
    // }

    if msg.attachments.len() < 1 && msg.embeds.len() < 1 {
        return Ok(());
    }

    // if msg.attachments.len() > 0 || msg.embeds.len() > 0 {
    //     // let role = db
    //     // .collection_allowed_role
    //     // .find_one(
    //     //     doc! { "guild_id": guild_id.to_string(), "id": role_option.id.to_string(), "feature": feature_option },
    //     //     None,
    //     // )
    //     // .await?;
    // }

    for attachment in msg.attachments.iter() {
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

    if !should_react {
        for embed in msg.embeds.iter() {
            let Some(img) = embed.image.as_ref() else {
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

    if should_react {
        let reactions = AutoReaction::to_reaction_type_vec(db.auto_reactions().await?)?;

        if reactions.len() < 1 {
            return Ok(());
        }

        let selected_reaction: usize = if reactions.len() == 1 {
            0
        } else {
            rand::thread_rng().gen_range(0..reactions.len())
        };
        let reaction = reactions
            .get(selected_reaction)
            .ok_or(HookReactError::GetReaction {
                selected: selected_reaction,
                total: reactions.len(),
            })?;
        msg.react(&ctx.http, reaction.to_owned().clone()).await?;

        // .collection_allowed_rol        let emoji = ctx.http.get_emoji(guild_id, emoji_id).await?;
        // msg.react(&ctx.http);
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum HookReactError {
    #[error("Serenity: {0}.")]
    Serenity(#[from] serenity::Error),

    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    #[error("ToReactionTypeError: {0}.")]
    ToReactionTypeError(#[from] ToReactionTypeError),

    #[error("Failed to get reaction by index: {selected:?} out of {total:?}.")]
    GetReaction { selected: usize, total: usize },
}
