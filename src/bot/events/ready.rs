use serenity::client::Context;
use serenity::model::prelude::GuildId;
use crate::bot::commands;

pub async fn ready(ctx: Context, ready: serenity::model::gateway::Ready) {
    println!("{} is connected!", ready.user.name);

    let db = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<crate::database::DB>()
            .expect("Expected crate::database::DB in TypeMap")
            .clone()
    };

    for guild in ctx.cache.guilds() {
        if !db.allowed_guild_exists(guild.0.to_string().as_str()).await.expect("Failed to read database.") {
            println!("Skipped command update for guild: {}", guild.0);
            continue;
        }

        let _commands = GuildId::set_application_commands(&guild, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::who::register(command))
                .create_application_command(|command| commands::verify::register(command))
                .create_application_command(|command| commands::test::register(command))
                .create_application_command(|command| commands::guilds::register(command))
                .create_application_command(|command| commands::leave::register(command))
                .create_application_command(|command| commands::sync::register(command))
                .create_application_command(|command| commands::add_channel::register(command))
                .create_application_command(|command| commands::add_role::register(command))
                .create_application_command(|command| commands::remove_guild::register(command))
                .create_application_command(|command| commands::remove_auto_emoji::register(command))
                .create_application_command(|command| commands::reset_time::register(command))
                .create_application_command(|command| commands::add_guild::register(command))
                .create_application_command(|command| commands::show_guilds::register(command))
                .create_application_command(|command| commands::remove_role::register(command))
                .create_application_command(|command| commands::add_auto_emoji::register(command))
                // .create_application_command(|command| commands::sync::register(command))
                .create_application_command(|command| {
                    commands::remove_channel::register(command)
                })
                .create_application_command(|command| {
                    commands::show_channels::register(command)
                })
                .create_application_command(|command| commands::show_roles::register(command))
        })
            .await;
        println!("Commands updated for guild id: {}", &guild);
    }
}