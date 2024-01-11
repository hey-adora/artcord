use crate::database::create_database::DB;
use serenity::client::Context;
use serenity::model::id::{ChannelId, GuildId, MessageId};

pub async fn message_delete(
    ctx: Context,
    _channel_id: ChannelId,
    deleted_message_id: MessageId,
    guild_id: Option<GuildId>,
) {
    let Some(guild_id) = guild_id else {
        return;
    };

    let db = {
        let data_read = ctx.data.read().await;

        data_read
            .get::<DB>()
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

    let result = db.img_hide(guild_id.0, deleted_message_id.0).await;

    let Ok(result) = result else {
        println!(
            "ERROR: failed to hide img '{}': {}",
            deleted_message_id.0,
            result.err().unwrap()
        );
        return;
    };

    if result {
        println!("IMG HIDDEN: {}", deleted_message_id);
    }
}
