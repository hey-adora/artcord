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

use crate::{
    bot::ReactionQueue,
    database::{AutoReaction, FromReactionTypeError, ToReactionTypeError, DB},
};

use super::save_attachments::SaveAttachmentsError;
use serenity::model::channel::Reaction;
use std::sync::Arc;

const CONFIRM_REACTION: &str = "✅";
const CLOSE_REACTION: &str = "❌";

pub async fn hook_add_reaction(
    ctx: &Context,
    remove: bool,
    guild_id: u64,
    reaction: &Reaction,
    db: &DB,
) -> Result<(), HookAddAutoReactionError> {
    let msg_id = reaction.message_id.0;
    let channel_id = reaction.channel_id.0;
    let reaction_queue_handle = {
        let data_read = ctx.data.read().await;

        data_read
            .get::<ReactionQueue>()
            .expect("Expected TypeMap")
            .clone()
    };

    println!("yo");

    {
        let reaction_queue: RwLockReadGuard<Option<ReactionQueue>> =
            reaction_queue_handle.read().await;
        println!("yo1");
        let Some(ref queue) = *reaction_queue else {
            println!("one");
            return Ok(());
        };

        // if let Some(a) = reaction_queue {}
        if queue.msg_id != msg_id {
            println!("two");
            return Ok(());
        }
    }

    println!("yo2");
    let mut reaction_queue: RwLockWriteGuard<Option<ReactionQueue>> =
        reaction_queue_handle.write().await;
    println!("yo3");
    let Some(ref mut queue) = *reaction_queue else {
        println!("three");
        return Ok(());
    };
    println!("bro");

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
                        db.auto_reactoin_insert_many_from_type(queue.reactions.clone())
                            .await?;
                    }
                    println!("Reactions: {:#?}", &queue.reactions);
                    *reaction_queue = None;
                    let _ = &ctx.http.delete_message(channel_id, msg_id).await?;
                }
                CLOSE_REACTION => {
                    println!("Reactions: {:#?}", &queue.reactions);
                    *reaction_queue = None;
                    let _ = &ctx.http.delete_message(channel_id, msg_id).await?;
                }
                wild => {
                    // let reaction_type = reaction.emoji.clone();
                    let auto_reaction =
                        AutoReaction::from_reaction_type(guild_id, reaction.emoji.clone())?;
                    let exists = db.auto_reactoin_exists(&auto_reaction).await?;
                    if !exists {
                        if remove {
                            queue.reactions = queue
                                .reactions
                                .clone()
                                .into_iter()
                                .filter(|r| {
                                    r.name != auto_reaction.name
                                        && r.animated != auto_reaction.animated
                                        && r.id != auto_reaction.id
                                        && r.unicode != auto_reaction.unicode
                                        && r.guild_id != auto_reaction.guild_id
                                })
                                .collect();
                        } else {
                            queue.reactions.push(auto_reaction);
                        }
                    }
                    println!("Reactions: {:#?}", &queue.reactions);
                }
            }
        }
        ReactionType::Custom { animated, id, name } => {
            let auto_reaction = AutoReaction::from_reaction_type(guild_id, reaction.emoji.clone())?;
            let exists = db.auto_reactoin_exists(&auto_reaction).await?;
            if !exists {
                if remove {
                    queue.reactions = queue
                        .reactions
                        .clone()
                        .into_iter()
                        .filter(|r| {
                            r.name != auto_reaction.name
                                && r.animated != auto_reaction.animated
                                && r.id != auto_reaction.id
                                && r.unicode != auto_reaction.unicode
                                && r.guild_id != auto_reaction.guild_id
                        })
                        .collect();
                } else {
                    queue.reactions.push(auto_reaction);
                }
            }
            println!("Reactions: {:#?}", &queue.reactions);
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
