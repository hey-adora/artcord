use std::sync::Arc;

use artcord_mongodb::database::DB;
use artcord_state::message::prod_server_msg::{ServerMsg};

use super::ResErr;


pub async fn user(db: Arc<DB>, user_id: String) -> Result<Option<ServerMsg>, ResErr> {
    Ok(Some(
        db.user_find_one(&user_id)
            .await?
            .map(|v| ServerMsg::User(Some(v)))
            .unwrap_or(ServerMsg::User(None)),
    ))
}


pub async fn user_gallery(
    db: Arc<DB>,
    amount: u32,
    from: i64,
    user_id: String,
) -> Result<Option<ServerMsg>, ResErr> {
    let result = db
        .img_aggregate_user_gallery(amount, from, &user_id)
        .await?;

    let Some(result) = result else {
        let res = ServerMsg::GalleryUser(None);
        return Ok(Some(res));
    };

    let res = ServerMsg::GalleryUser(Some(result));
    Ok(Some(res))
}
