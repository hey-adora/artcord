use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, EmojiId, InteractionResponseType,
    },
    prelude::Context,
};

use crate::{bot::ReactionQueue, database::DB};

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
    let mut reaction_queue = reaction_queue.write().await;

    let mut output = String::from("Loading...");
    let a = command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await?;

    // let result = command
    //     .channel_id
    //     .send_message(&ctx.http, |msg| {
    //         msg.content("React to this message to add auto emoji.")
    //     })
    //     .await?;
    let msg = command.get_interaction_response(&ctx.http).await?;
    *reaction_queue = Some(ReactionQueue::new(msg.id.0, true));

    // let a = msg.react(&ctx.http, EmojiId()).await?;

    command
        .edit_original_interaction_response(&ctx.http, |message| {
            message.content(format!(
                "React to this message to add auto emoji. {}",
                msg.id.0
            ))
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("add_auto_emoji")
        .description("React to add emoji.")
}
