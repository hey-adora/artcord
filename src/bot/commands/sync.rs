use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;

use crate::bot::commands::{get_option_channel, get_option_integer};
use crate::database::DB;

pub async fn run(
    options: &[CommandDataOption],
    db: &DB,
    guild_id: u64,
) -> Result<String, crate::bot::commands::CommandError> {
    let channel_option = get_option_channel(options.get(0))?;
    let amount_option = get_option_integer(options.get(1))?;

    println!("hello");

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
                .description(format!("Amount from 1 to {}.", i32::MAX))
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .max_int_value(i32::MAX)
                .required(true)
        })
}
