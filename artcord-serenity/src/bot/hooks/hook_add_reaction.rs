use async_std::sync::RwLock;
use bson::doc;
use rand::Rng;
use std::path::Path;
use thiserror::Error;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

use serenity::{
    model::{
        channel::Message,
        prelude::{Emoji, ReactionType},
    },
    prelude::Context,
};

use super::save_attachments::SaveAttachmentsError;
use crate::bot::create_bot::ReactionQueue;
use crate::database::create_database::DB;
use crate::database::models::auto_reaction::{
    AutoReaction, FromReactionTypeError, ToReactionTypeError,
};
use serenity::model::channel::Reaction;
use std::sync::Arc;

pub const CONFIRM_REACTION: &str = "✅";
pub const CLOSE_REACTION: &str = "❌";

pub async fn hook_add_reaction(
    ctx: &Context,
    remove: bool,
    guild_id: u64,
    reaction: &Reaction,
    db: &DB,
) -> Result<(), HookAddAutoReactionError> {
    let msg_id = reaction.message_id.0;
    let channel_id = reaction.channel_id.0;
    let Some(member) = &reaction.member else {
        return Ok(());
    };

    let Some(user) = &member.user else {
        return Ok(());
    };

    if user.bot {
        return Ok(());
    }

    let reaction_queue_handle = {
        let data_read = ctx.data.read().await;

        data_read
            .get::<ReactionQueue>()
            .expect("Expected TypeMap")
            .clone()
    };

    {
        let reaction_queue = reaction_queue_handle.read().await;

        let Some(ref queue) = reaction_queue.get(&guild_id) else {
            return Ok(());
        };

        if queue.msg_id != msg_id {
            return Ok(());
        }
    }

    let mut reaction_queue_map = reaction_queue_handle.write().await;

    let Some(ref mut queue) = reaction_queue_map.get_mut(&guild_id) else {
        return Ok(());
    };

    match &reaction.emoji {
        ReactionType::Unicode(reaction_string) => {
            let reaction_str = reaction_string.as_str();
            match reaction_str {
                CONFIRM_REACTION => {
                    if queue.reactions.len() > 0 {
                        // let auto_reactions = AutoReaction::from_reaction_type_vec(
                        //     guild_id,
                        //     queue.reactions.to_owned(),
                        // )?;
                        if queue.add {
                            db.auto_reactoin_insert_many_from_type(queue.reactions.clone())
                                .await?;
                        } else {
                            db.auto_reactoin_delete_many(queue.reactions.clone())
                                .await?;
                        }
                    }
                    // println!("Reactions: {:#?}", &queue.reactions);
                    reaction_queue_map.remove(&guild_id);
                    let _ = &ctx.http.delete_message(channel_id, msg_id).await?;
                }
                CLOSE_REACTION => {
                    // println!("Reactions: {:#?}", &queue.reactions);
                    reaction_queue_map.remove(&guild_id);
                    let _ = &ctx.http.delete_message(channel_id, msg_id).await?;
                }
                wild => {
                    // let reaction_type = reaction.emoji.clone();
                    let auto_reaction =
                        AutoReaction::from_reaction_type(guild_id, reaction.emoji.clone())?;
                    let exists = db.auto_reactoin_exists(&auto_reaction).await?;
                    if (!exists && queue.add) || (exists && !queue.add) {
                        // if remove {
                        //     queue.reactions = queue
                        //         .reactions
                        //         .clone()
                        //         .into_iter()
                        //         .filter(|r| {
                        //             r.name != auto_reaction.name
                        //                 && r.animated != auto_reaction.animated
                        //                 && r.id != auto_reaction.id
                        //                 && r.unicode != auto_reaction.unicode
                        //                 && r.guild_id != auto_reaction.guild_id
                        //         })
                        //         .collect();
                        // } else {
                        //     queue.reactions.push(auto_reaction);
                        // }
                        queue.reactions.push(auto_reaction);
                    }
                    println!("Reactions: {:#?}", &queue.reactions);
                }
            }
        }
        ReactionType::Custom { animated, id, name } => {
            let auto_reaction = AutoReaction::from_reaction_type(guild_id, reaction.emoji.clone())?;
            let exists = db.auto_reactoin_exists(&auto_reaction).await?;
            if (!exists && queue.add) || (exists && !queue.add) {
                // if remove {
                //     queue.reactions = queue
                //         .reactions
                //         .clone()
                //         .into_iter()
                //         .filter(|r| {
                //             r.name != auto_reaction.name
                //                 && r.animated != auto_reaction.animated
                //                 && r.id != auto_reaction.id
                //                 && r.unicode != auto_reaction.unicode
                //                 && r.guild_id != auto_reaction.guild_id
                //         })
                //         .collect();
                // } else {
                //     queue.reactions.push(auto_reaction);
                // }
                queue.reactions.push(auto_reaction);
            }
            // println!("Reactions: {:#?}", &queue.reactions);
        }
        _ => {
            Err(HookAddAutoReactionError::InvalidEmoji)?;
        }
    }

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
