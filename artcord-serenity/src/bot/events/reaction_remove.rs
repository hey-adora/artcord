use crate::bot::create_bot::ReactionQueue;
use crate::bot::hooks::hook_add_reaction::hook_add_reaction;
use crate::database::create_database::DB;
use serenity::client::Context;
use serenity::model::channel::Reaction;

pub async fn reaction_remove(ctx: Context, remove_reaction: Reaction) {
    let Some(guild_id) = remove_reaction.guild_id else {
        return;
    };

    let (db, reaction_queue) = {
        let data_read = ctx.data.read().await;

        let db = data_read
            .get::<DB>()
            .expect("Expected crate::database::DB in TypeMap")
            .clone();
        let reaction_queue = data_read
            .get::<ReactionQueue>()
            .expect("Expected crate::database::DB in TypeMap")
            .clone();
        (db, reaction_queue)
    };

    let allowed_guild = db
        .allowed_guild_exists(guild_id.0.to_string().as_str())
        .await;
    let Ok(allowed_guild) = allowed_guild else {
        println!("Mongodb error: {}", allowed_guild.err().unwrap());
        return;
    };
    if !allowed_guild {
        return;
    }

    let result = hook_add_reaction(&ctx, true, guild_id.0, &remove_reaction, &db).await;

    if let Err(err) = result {
        println!("{:?}", err);
        return;
    }
}
