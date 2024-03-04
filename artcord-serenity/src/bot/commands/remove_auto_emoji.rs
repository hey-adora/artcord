use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, EmojiId, InteractionResponseType,
        ReactionType,
    },
    prelude::Context,
};

use crate::bot::create_bot::ReactionQueue;
use crate::bot::hooks::hook_add_reaction::{CLOSE_REACTION, CONFIRM_REACTION};
use crate::database::create_database::DB;

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
    guild_id: u64,
) -> Result<(), crate::bot::commands::CommandError> {
    let reaction_queue = {
        let data_read = ctx.data.read().await;

        data_read
            .get::<ReactionQueue>()
            .expect("Expected TypeMap")
            .clone()
    };
    let mut reaction_queue_map = reaction_queue.write().await;
    let mut reaction_queue = reaction_queue_map.get_mut(&guild_id);

    if let Some(reaction_queue) = reaction_queue {
        ctx.http
            .delete_message(reaction_queue.channel_id, reaction_queue.msg_id)
            .await?;
    }

    // let mut output = String::from("Loading...");
    command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await?;

    let msg = command.get_interaction_response(&ctx.http).await?;

    msg.react(
        &ctx.http,
        ReactionType::Unicode(CONFIRM_REACTION.to_string()),
    )
    .await?;

    msg.react(&ctx.http, ReactionType::Unicode(CLOSE_REACTION.to_string()))
        .await?;

    let reactions = db.auto_reactions(guild_id).await?;

    for reaction in reactions {
        msg.react(&ctx.http, reaction.to_reaction_type()?).await?;
    }

    reaction_queue_map.insert(
        guild_id,
        ReactionQueue::new(msg.channel_id.0, msg.id.0, false),
    );

    command
        .edit_original_interaction_response(&ctx.http, |message| {
            message.content(format!(
                "React to this message to remove auto emoji. {}",
                msg.id.0
            ))
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("remove_auto_emoji")
        .description("React to remove emoji.")
}
