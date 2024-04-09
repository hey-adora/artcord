use std::sync::Arc;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::{
    prod_perm_key::ProdMsgPermKey,
    prod_server_msg::{ServerMsg, UserGalleryResponse},
};
use thiserror::Error;

use crate::ws_app::WsResError;

pub async fn ws_handle_user_gallery(
    db: Arc<DB>,
    amount: u32,
    from: i64,
    user_id: String,
) -> Result<ServerMsg, WsResError> {
    let result = db
        .img_aggregate_user_gallery(amount, from, &user_id)
        .await?;

    let Some(result) = result else {
        let res = UserGalleryResponse::UserNotFound;
        let res = ServerMsg::UserGallery(res);
        return Ok(res);
    };

    let res = UserGalleryResponse::Imgs(result);
    let res = ServerMsg::UserGallery(res);
    Ok(res)
}

#[derive(Error, Debug)]
pub enum WsHandleUserGalleryError {
    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),
}

// enum User {
//     Stuff
// }

// enum Req {
//     User(User)
// }

// fn test () {
//     let wow = Req::User(User::Stuff);
// }

