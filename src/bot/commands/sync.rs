use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;

use crate::database::DB;

pub async fn run(options: &[CommandDataOption], db: &DB) -> String {
    let Some(option) = options.get(0) else {
        return "Sync amount option was not provided.".to_string();
    };

    let Some(option) = option.resolved.as_ref() else {
        return "Sync amount option was not provided.".to_string();
    };

    let CommandDataOptionValue::Integer(option) = option else {
        return "Sync amount option was not provided.".to_string();
    };
    //let option = CommandDataOptionValue::Integer(l);

    println!("{:#?}", option);

    String::from("Test")
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("sync")
        .description("Upload images from specific channel")
        .create_option(|option| {
            option
                .name("int")
                .description(format!("Amount from 1 to {}", i32::MAX))
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .max_int_value(i32::MAX)
                .required(true)
        })
}
