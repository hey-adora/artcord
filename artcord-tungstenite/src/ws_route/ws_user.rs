use std::sync::Arc;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::{
    prod_perm_key::ProdMsgPermKey,
    prod_server_msg::{ServerMsg, UserResponse},
};
use thiserror::Error;

use crate::ws_app::WsResError;

pub async fn ws_handle_user(db: Arc<DB>, user_id: String) -> Result<ServerMsg, WsResError> {
    let result = db.user_find_one(&user_id).await?;

    let Some(result) = result else {
        let res = UserResponse::UserNotFound;
        let res = ServerMsg::User(res);
        return Ok(res);
    };
    let res = UserResponse::User(result);
    let res = ServerMsg::User(res);
    Ok(res)
}

// #[derive(Error, Debug)]
// pub enum WsHandleUserError {
//     #[error("Mongodb error: {0}")]
//     MongoDB(#[from] mongodb::error::Error),
// }

// enum User {
//     Stuff
// }

// enum Req {
//     User(User)
// }

// fn test () {
//     let wow = Req::User(User::Stuff);
// }
