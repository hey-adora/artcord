use serenity::builder::CreateApplicationCommand;
use serenity::http::Http;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::ChannelId;
use serenity::prelude::Context;

use crate::bot::commands::{get_option_channel, get_option_integer};
use crate::bot::hooks::save_attachments::{self, hook_save_attachments};
use crate::database::DB;

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
    guild_id: u64,
) -> Result<(), crate::bot::commands::CommandError> {
    let channel_option = get_option_channel(command.data.options.get(0))?;
    let amount_option = get_option_integer(command.data.options.get(1))?;

    let messages = command
        .channel_id
        .messages(ctx.http.as_ref(), |f| f.limit((*amount_option) as u64))
        .await?;

    let len = messages.len();

    for message in messages {
        // println!("attachments len: {}", message.attachments.len());
        hook_save_attachments(
            &message.attachments,
            db,
            guild_id,
            command.channel_id.0,
            message.id.0,
            message.author.id.0,
            message.author.name,
            message.author.avatar,
            true,
        )
        .await?;
    }
    // let messages = http.get_messages(channel_id, "").await?;

    // println!("{}", );

    // Ok(format!("Synced {}", len))
    Ok(())
}

// pub async fn run_update(
//     options: &[CommandDataOption],
//     db: &DB,
//     guild_id: u64,
//     channel_id: ChannelId,
//     http: &Http,
//     command: ApplicationCommandInteraction
// ) -> Result<String, crate::bot::commands::CommandError> {
//
// }

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
