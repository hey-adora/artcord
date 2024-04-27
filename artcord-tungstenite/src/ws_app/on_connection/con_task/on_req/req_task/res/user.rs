use std::sync::Arc;

use artcord_mongodb::database::DB;
use artcord_state::message::prod_server_msg::{ServerMsg};

use crate::ws_app::WsResError;

pub async fn ws_handle_user(db: Arc<DB>, user_id: String) -> Result<Option<ServerMsg>, WsResError> {
    Ok(Some(
        db.user_find_one(&user_id)
            .await?
            .map(|v| ServerMsg::User(Some(v)))
            .unwrap_or(ServerMsg::User(None)),
    ))
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
