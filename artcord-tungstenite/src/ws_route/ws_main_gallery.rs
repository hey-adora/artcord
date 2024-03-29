use std::sync::Arc;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::{prod_perm_key::ProdMsgPermKey, prod_server_msg::ServerMsg};
use thiserror::Error;

pub async fn ws_handle_main_gallery(db: Arc<DB>, amount: u32, from: i64) -> Result<artcord_state::message::prod_server_msg::MainGalleryResponse, WsHandleMainGalleryError> {
    let result = db.img_aggregate_gallery(amount, from)
    .await?;

    // let server_package = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
    //     key,
    //     data: ServerMsg::MainGallery(artcord_state::message::prod_server_msg::MainGalleryResponse::Imgs(result)),
    // };
    // Ok(server_package)
    Ok(artcord_state::message::prod_server_msg::MainGalleryResponse::Imgs(result))
}

#[derive(Error, Debug)]
pub enum WsHandleMainGalleryError {
    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),

    // #[error("Bcrypt error: {0}")]
    // Bcrypt(#[from] bcrypt::BcryptError),

    // #[error("JWT error: {0}")]
    // JWT(#[from] jsonwebtoken::errors::Error),

    // #[error("RwLock error: {0}")]
    // RwLock(String),
}
