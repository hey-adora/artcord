use std::sync::Arc;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::{
    prod_perm_key::ProdMsgPermKey,
    prod_server_msg::{ServerMsg, UserRes},
};
use thiserror::Error;

use crate::ws_app::WsResError;

pub async fn ws_handle_user(db: Arc<DB>, user_id: String) -> Result<Option<ServerMsg>, WsResError> {
    let result = db.user_find_one(&user_id).await?;

    let Some(result) = result else {
        let res = UserRes::UserNotFound;
        let res = ServerMsg::User(res);
        return Ok(Some(res));
    };
    let res = UserRes::User(result);
    let res = ServerMsg::User(res);
    Ok(Some(res))
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
