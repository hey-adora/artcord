use crate::create_bot::ArcDB;
use crate::hooks::hook_add_reaction::hook_add_reaction;
use artcord_mongodb::database::DB;
use serenity::client::Context;
use serenity::model::channel::Reaction;

pub async fn reaction_add(ctx: Context, add_reaction: Reaction) {
    let Some(guild_id) = add_reaction.guild_id else {
        return;
    };

    let db = {
        let data_read = ctx.data.read().await;

        data_read
            .get::<ArcDB>()
            .expect("Expected crate::database::DB in TypeMap")
            .clone()
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

    let result = hook_add_reaction(&ctx, false, guild_id.0, &add_reaction, &db).await;

    if let Err(err) = result {
        println!("{:?}", err);
        return;
    }
}
