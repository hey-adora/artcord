use artcord_mongodb::database::DB;
use serenity::client::Context;
use serenity::model::id::{ChannelId, GuildId, MessageId};

use crate::create_bot::ArcDB;

pub async fn message_delete_bulk(
    ctx: Context,
    _channel_id: ChannelId,
    multiple_deleted_messages_id: Vec<MessageId>,
    guild_id: Option<GuildId>,
) {
    let Some(guild_id) = guild_id else {
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

    for deleted_message_id in multiple_deleted_messages_id {
        let result = db.img_hide(guild_id.0, deleted_message_id.0).await;
        // let result = db
        //     .collection_img
        //     .update_one(
        //         doc! { "id": deleted_message_id.0.to_string() },
        //         doc! { "$set": { "show": false } },
        //         None,
        //     )
        //     .await;
        let Ok(_) = result else {
            println!(
                "ERROR: failed to hide img '{}': {}",
                deleted_message_id.0,
                result.err().unwrap()
            );
            return;
        };

        println!("IMG HIDDEN: {}", deleted_message_id);
    }
}
