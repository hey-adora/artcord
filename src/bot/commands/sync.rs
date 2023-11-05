use serenity::builder::CreateApplicationCommand;
use serenity::http::Http;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::ChannelId;
use serenity::prelude::Context;

use crate::bot::commands::{get_option_channel, get_option_integer};
use crate::database::DB;

pub async fn run(
    options: &[CommandDataOption],
    db: &DB,
    guild_id: u64,
    channel_id: ChannelId,
    http: &Http,
) -> Result<String, crate::bot::commands::CommandError> {
    let channel_option = get_option_channel(options.get(0))?;
    let amount_option = get_option_integer(options.get(1))?;

    let messages = channel_id
        .messages(http, |f| f.limit((*amount_option) as u64))
        .await?;
    // let messages = http.get_messages(channel_id, "").await?;

    println!("{}", messages.len());

    Ok(String::from("Test"))
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("sync")
        .description("Upload images from specific channel")
        .create_option(|option| {
            option
                .name("channel")
                .description(format!("Channel to sync images from."))
                .kind(CommandOptionType::Channel)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("amount")
                .description(format!("Amount from 1 to {}.", 100))
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .max_int_value(100)
                .required(true)
        })
}
