use std::sync::Arc;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::{prod_perm_key::ProdMsgPermKey, prod_server_msg::ServerMsg};
use thiserror::Error;

pub async fn ws_handle_user(
    db: Arc<DB>,
    user_id: String,
) -> Result<artcord_state::message::prod_server_msg::UserResponse, WsHandleUserError> {
    let result = db.user_find_one(&user_id).await?;

    let Some(result) = result else {
        return Ok(artcord_state::message::prod_server_msg::UserResponse::UserNotFound);
    };

    Ok(artcord_state::message::prod_server_msg::UserResponse::User(result))
}

#[derive(Error, Debug)]
pub enum WsHandleUserError {
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
