use serenity::builder::CreateApplicationCommand;
use serenity::http::Http;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::{ChannelId, InteractionResponseType};
use serenity::prelude::Context;

use crate::bot::commands::{get_option_channel, get_option_integer};
use crate::bot::hooks::save_attachments::{self, hook_save_attachments};
use crate::database::DB;

use super::CommandError;

pub const DISCORD_MAX_MSG_REQUEST_SIZE: i64 = 100;

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &DB,
    guild_id: u64,
) -> Result<(), crate::bot::commands::CommandError> {
    let channel_option = get_option_channel(command.data.options.get(0))?;
    let mut amount_option = *get_option_integer(command.data.options.get(1))?;

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }

    // return Err(CommandError::NotFound("yoyoyyomf".to_string()));

    let mut total_synced: usize = 0;

    if amount_option < 1 {
        amount_option = i64::MAX;
    }
    let mut amount_fraction: i64 = amount_option % DISCORD_MAX_MSG_REQUEST_SIZE;
    if amount_fraction == 0 {
        amount_fraction = DISCORD_MAX_MSG_REQUEST_SIZE;
    }
    let loop_amount: i64 = amount_option - amount_fraction;

    let messages = command
        .channel_id
        .messages(ctx.http.as_ref(), |f| f.limit(amount_fraction as u64))
        .await?;

    total_synced += messages.len();
    command
        .edit_original_interaction_response(&ctx.http, |message| {
            message.content(format!("Syncing... {}/{}", total_synced, amount_option))
        })
        .await?;
    // update_status.await;

    for message in &messages {
        let result = hook_save_attachments(
            &message.attachments,
            db,
            guild_id,
            command.channel_id.0,
            message.id.0,
            message.author.id.0,
            message.author.name.clone(),
            message.author.avatar.clone(),
            true,
        )
        .await;

        // if let Err(err) = result {
        //
        // }
        match result {
            Ok(_) => Ok(()),
            Err(err) => match err {
                save_attachments::SaveAttachmentsError::SaveAttachmentError(e) => match e {
                    save_attachments::SaveAttachmentError::ImgTypeUnsupported(r) => {
                        println!("UNSUPPORTED IMG: {}", r);
                        Ok(())
                    }
                    r => Err(save_attachments::SaveAttachmentsError::from(r)),
                },
                r => Err::<(), save_attachments::SaveAttachmentsError>(r),
            },
        }?;
    }

    if messages.len() < amount_fraction as usize {
        command
            .edit_original_interaction_response(&ctx.http, |message| {
                message.content(format!(
                    "Syncing complete. {}/{}",
                    total_synced, amount_option
                ))
            })
            .await?;
        return Ok(());
    }

    let Some(last) = messages.last() else {
        command
            .edit_original_interaction_response(&ctx.http, |message| {
                message.content(format!(
                    "Syncing complete. {}/{}",
                    total_synced, amount_option
                ))
            })
            .await?;
        return Ok(());
    };

    let mut msg = (*last).clone();

    for i in (0..loop_amount).step_by(DISCORD_MAX_MSG_REQUEST_SIZE as usize) {
        let messages = command
            .channel_id
            .messages(ctx.http.as_ref(), |f| {
                f.before(msg.id).limit(DISCORD_MAX_MSG_REQUEST_SIZE as u64)
            })
            .await?;

        for message in &messages {
            let result = hook_save_attachments(
                &message.attachments,
                db,
                guild_id,
                command.channel_id.0,
                message.id.0,
                message.author.id.0,
                message.author.name.clone(),
                message.author.avatar.clone(),
                true,
            )
            .await;

            match result {
                Ok(_) => Ok(()),
                Err(err) => match err {
                    save_attachments::SaveAttachmentsError::SaveAttachmentError(e) => match e {
                        save_attachments::SaveAttachmentError::ImgTypeUnsupported(r) => {
                            println!("UNSUPPORTED IMG: {}", r);
                            Ok(())
                        }
                        r => Err(save_attachments::SaveAttachmentsError::from(r)),
                    },
                    r => Err::<(), save_attachments::SaveAttachmentsError>(r),
                },
            }?;
        }

        total_synced += messages.len();

        if messages.len() < amount_fraction as usize {
            break;
        }

        let Some(new_last) = messages.last() else {
            break;
        };

        command
            .edit_original_interaction_response(&ctx.http, |message| {
                message.content(format!("Syncing... {}/{}", total_synced, amount_option))
            })
            .await?;

        msg = (*new_last).clone();
    }

    command
        .edit_original_interaction_response(&ctx.http, |message| {
            message.content(format!(
                "Syncing complete. {}/{}",
                total_synced, amount_option
            ))
        })
        .await?;

    Ok(())
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
                .description(format!("Amount from 0 to {}.", i32::MAX))
                .kind(CommandOptionType::Integer)
                .min_int_value(0)
                .max_int_value(i32::MAX)
                .required(true)
        })
}
