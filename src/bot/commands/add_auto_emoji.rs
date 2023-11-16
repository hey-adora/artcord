use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{application_command::ApplicationCommandInteraction, InteractionResponseType},
    prelude::Context,
};

use crate::database::DB;

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
    guild_id: u64,
) -> Result<(), crate::bot::commands::CommandError> {
    // let role = db
    //     .collection_allowed_role
    //     .find_one(
    //         doc! { "guild_id": guild_id.to_string(), "id": role_option.id.to_string(), "feature": feature_option },
    //         None,
    //     )
    //     .await?;
    let mut output = String::from("Loading...");
    let a = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(output))
        })
        .await?;

    let result = command
        .channel_id
        .send_message(&ctx.http, |msg| {
            msg.content("React to this message to add auto emoji.")
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("add_auto_emoji")
        .description("React to add emoji.")
}
