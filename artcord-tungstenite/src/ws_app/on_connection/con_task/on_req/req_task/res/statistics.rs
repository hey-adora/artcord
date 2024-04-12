use std::sync::Arc;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::{
    message::{prod_perm_key::ProdMsgPermKey, prod_server_msg::ServerMsg},
    model::statistics::Statistic,
};
use thiserror::Error;

pub async fn ws_statistics(db: Arc<DB>) -> Result<Vec<Statistic>, WsStatisticsError> {
    // let result = db.user_find_one(&user_id).await?;
    //
    // let Some(result) = result else {
    //     return Ok(artcord_state::message::prod_server_msg::UserResponse::UserNotFound);
    // };
    //
    // Ok(artcord_state::message::prod_server_msg::UserResponse::User(
    //     result,
    // ))
    // let a = Statistic::new("127.0.0.1".to_string());
    Ok(vec![])
}

#[derive(Error, Debug)]
pub enum WsStatisticsError {
    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),
}
