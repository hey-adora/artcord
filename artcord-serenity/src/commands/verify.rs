use std::collections::HashMap;

use artcord_mongodb::database::DB;
use bson::doc;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        application::command::CommandOptionType,
        prelude::{application_command::ApplicationCommandInteraction, InteractionResponseType},
    },
    prelude::Context,
};

use super::get_option_integer;

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
) -> Result<(), crate::commands::CommandError> {
    let code = (get_option_integer(command.data.options.get(0))?
        .clone()
        .clamp(1000, 9999 + 1) as i32)
        .to_be_bytes()
        .to_vec();
    let user_id = command.user.id.0.to_be_bytes().to_vec();
    let msg = [user_id, code].concat();
    //let code = 10i32.to_be_bytes().to_vec();

    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:8069/verify")
        .body(msg)
        .send()
        .await?;
    // let guilds = db.allowed_guild_all().await?;

    // let mut output = String::from("Guilds:");

    // if guilds.len() < 1 {
    //     output.push_str(" none.");
    // }

    // for guild in guilds {
    //     output.push_str(&format!("\n-{}:{}", guild.id, guild.name));
    // }

    // command
    //     .create_interaction_response(&ctx.http, |response| {
    //         response
    //             .kind(InteractionResponseType::ChannelMessageWithSource)
    //             .interaction_response_data(|message| message.content(output))
    //     })
    //     .await?;

    let output = format!(
        "{}",
        match res.status().as_u16() {
            200 => "Verified.",
            401 => "Invalid discord account.",
            404 => "Failed to verify.",
            _ => "Failed to verify 500.",
        }
    );
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(output))
        })
        .await?;
    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("verify")
        .description("Verify to join minecraft server")
        .create_option(|option| {
            option
                .name("code")
                .description(format!(
                    "Code that is shown when you try to join minecraft server."
                ))
                .kind(CommandOptionType::Integer)
                .required(true)
                .min_int_value(1000)
                .max_int_value(9999)
        })
}
