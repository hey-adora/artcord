use bson::doc;
use rand::Rng;
use std::path::Path;
use thiserror::Error;

use serenity::{
    model::{
        channel::Message,
        prelude::{Emoji, ReactionType},
    },
    prelude::Context,
};

use crate::{
    bot::ReactionQueue,
    database::{AutoReaction, FromReactionTypeError, ToReactionTypeError, DB},
};

use super::save_attachments::SaveAttachmentsError;
use serenity::model::channel::Reaction;

const CONFIRM_REACTION: &str = "✅";
const CLOSE_REACTION: &str = "❌";

pub async fn hook_add_reaction(
    ctx: &Context,
    guild_id: u64,
    reaction: &Reaction,
    db: &DB,
) -> Result<(), HookAddAutoReactionError> {
    let msg_id = reaction.message_id.0;
    let channel_id = reaction.channel_id.0;
    let reaction_queue = {
        let data_read = ctx.data.read().await;

        data_read
            .get::<ReactionQueue>()
            .expect("Expected TypeMap")
            .clone()
    };
    let mut reaction_queue = reaction_queue.write().await;
    let Some(reactions) = reaction_queue.get_mut(&msg_id) else {
        return Ok(());
    };

    match &reaction.emoji {
        ReactionType::Unicode(reaction_string) => {
            let reaction_str = reaction_string.as_str();
            match reaction_str {
                CONFIRM_REACTION => {
                    if reactions.len() > 0 {
                        let auto_reactions =
                            AutoReaction::from_reaction_type(guild_id, reactions.to_owned())?;
                        db.auto_reactoin_insert_many_from_type(auto_reactions)
                            .await?;
                    }
                    println!("Reactions: {:#?}", &reactions);
                    reaction_queue.remove(&msg_id);
                    let _ = &ctx.http.delete_message(channel_id, msg_id).await?;
                }
                CLOSE_REACTION => {
                    println!("Reactions: {:#?}", &reactions);
                    reaction_queue.remove(&msg_id);
                    let _ = &ctx.http.delete_message(channel_id, msg_id).await?;
                }
                wild => {
                    let reaction_type = reaction.emoji.clone();
                    reactions.push(reaction_type);
                    println!("Reactions: {:#?}", &reactions);
                }
            }
        }
        ReactionType::Custom { animated, id, name } => {
            let reaction_type = reaction.emoji.clone();
            reactions.push(reaction_type);
            println!("Reactions: {:#?}", &reactions);
        }
        _ => {
            Err(HookAddAutoReactionError::InvalidEmoji)?;
        }
    }

    // if let serenity::model::prelude::ReactionType::Unicode(reaction) = &reaction.emoji {
    //     let reaction_str = reaction.as_str();
    //     if reaction_str == CONFIRM_REACTION {
    //         if reactions.len() > 0 {
    //             let auto_reactions =
    //                 AutoReaction::from_reaction_type(guild_id, reactions.to_owned())?;
    //             db.auto_reactoin_insert_many_from_type(auto_reactions)
    //                 .await?;
    //         }
    //         &ctx.http
    //             .delete_message(guild_id, reaction.message_id.0)
    //             .await?;
    //     }
    // }

    // match add_reaction.emoji {
    //     serenity::model::prelude::ReactionType::Unicode(s) => println!("Unicode: {}", s),
    //     serenity::model::prelude::ReactionType::Custom { animated, id, name } => {
    //         println!("Custom: {}", id)
    //     }
    //     _ => println!("wtf>>>"),
    // }

    Ok(())
}

#[derive(Error, Debug)]
pub enum HookAddAutoReactionError {
    #[error("Serenity: {0}.")]
    Serenity(#[from] serenity::Error),

    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    #[error("ToReactionTypeError: {0}.")]
    ToReactionTypeError(#[from] ToReactionTypeError),

    #[error("ToReactionTypeError: {0}.")]
    FromReactionTypeError(#[from] FromReactionTypeError),

    #[error("Invalid Emoji")]
    InvalidEmoji,

    #[error("Failed to get reaction by index: {selected:?} out of {total:?}.")]
    GetReaction { selected: usize, total: usize },
}
