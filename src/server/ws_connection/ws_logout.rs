use crate::database::create_database::DB;
use crate::database::models::acc::Acc;
use crate::server::server_msg::ServerMsg;
use crate::server::ws_connection::ServerMsgCreationError;
use std::sync::{Arc, Mutex, RwLock};

pub async fn ws_logout(acc: Arc<RwLock<Option<Acc>>>) -> Result<ServerMsg, ServerMsgCreationError> {
    let mut acc = acc
        .write()
        .or_else(|e| Err(ServerMsgCreationError::RwLock(e.to_string())))?;
    *acc = None;

    Ok(ServerMsg::LoggedOut)
}
