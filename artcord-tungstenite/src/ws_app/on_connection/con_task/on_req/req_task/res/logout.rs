use artcord_mongodb::database::DB;
use artcord_state::{message::prod_server_msg::ServerMsg, model::acc::Acc};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::ws_app::WsResError;

pub async fn ws_logout(acc: Arc<RwLock<Option<Acc>>>) -> Result<ServerMsg, WsResError> {
    let mut acc = acc.write().await;

    *acc = None;

    Ok(ServerMsg::LoggedOut)
}
