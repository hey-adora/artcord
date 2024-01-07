use serenity::client::Context;
use serenity::model::channel::ReactionType;
use serenity::model::id::EmojiId;
use crate::bot::create_bot::ArcStr;
use crate::bot::hooks::hook_auto_reaction::hook_auto_react;
use crate::bot::hooks::save_attachments::hook_save_attachments;

pub async fn message( ctx: Context, msg: serenity::model::channel::Message) {
    let Some(guild_id) = msg.guild_id else {
        return;
    };

    let time = msg.timestamp.timestamp_millis();

    let (db, gallery_root_dir) = {
        let data_read = ctx.data.read().await;

        let db = data_read
            .get::<crate::database::DB>()
            .expect("Expected crate::database::DB in TypeMap")
            .clone();
        let gallery_root_dir = data_read
            .get::<ArcStr>()
            .expect("Expected crate::database::DB in TypeMap")
            .clone();
        (db, gallery_root_dir)
    };

    let allowed_guild = db.allowed_guild_exists(guild_id.0.to_string().as_str()).await;
    let Ok(allowed_guild) = allowed_guild else {
        println!("Mongodb error: {}", allowed_guild.err().unwrap());
        return;
    };
    if !allowed_guild {
        return;
    }

    let result = hook_save_attachments(
        &*gallery_root_dir,
        &msg.attachments,
        &db,
        time,
        guild_id.0,
        msg.channel_id.0,
        msg.id.0,
        msg.author.id.0,
        msg.author.name.clone(),
        msg.author.avatar.clone(),
        false,
    )
        .await;

    if let Err(err) = result {
        println!("{:?}", err);
        return;
    }

    let a = msg.react(&ctx.http, ReactionType::Custom { animated: false, id: EmojiId(1175429915999490152), name: Some(String::from("done")) }).await;

    let result = hook_auto_react(&ctx, guild_id.0, &msg, &db, false).await;

    if let Err(err) = result {
        println!("{:?}", err);
        return;
    }
}