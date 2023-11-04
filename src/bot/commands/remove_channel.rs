use bson::doc;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{CommandDataOption, CommandDataOptionValue},
        command::CommandOptionType,
    },
};

use crate::database::{AllowedChannel, DB};

use super::{get_option_channel, get_option_string, Feature};

pub async fn run(
    options: &[CommandDataOption],
    db: &DB,
) -> Result<String, crate::bot::commands::CommandError> {
    let channel_option = get_option_channel(options.get(0))?;
    let feature_option = get_option_string(options.get(1))?;

    Feature::is_valid(feature_option)?;

    let result = db
        .collection_allowed_channel
        .delete_one(
            doc! { "id": channel_option.id.0.to_string(), "feature": feature_option },
            None,
        )
        .await?;

    if result.deleted_count < 1 {
        return Err(crate::bot::commands::CommandError::NotFound(format!(
            "feature: {} in <#{}>",
            feature_option, channel_option.id
        )));
    } else {
        return Ok(format!(
            "feature: {} in <#{}> was removed.",
            feature_option, channel_option.id
        ));
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("remove_channel")
        .description("Remove channel to whitelist of specific feature")
        .create_option(|option| {
            option
                .name("channel")
                .description(format!("Channel to remove."))
                .kind(CommandOptionType::Channel)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("feature")
                .description(format!("Features: gallery."))
                .kind(CommandOptionType::String)
                .required(true)
        })
}
