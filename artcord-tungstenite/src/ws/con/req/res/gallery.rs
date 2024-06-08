use std::sync::Arc;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::global;
use thiserror::Error;

use super::ResErr;


pub async fn gallery(
    db: Arc<DB>,
    amount: u32,
    from: i64,
) -> Result<Option<global::ServerMsg>, ResErr> {
    let result = db.img_aggregate_gallery(amount, from).await?;

    // let server_package = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
    //     key,
    //     data: ServerMsg::MainGallery(artcord_state::message::prod_server_msg::MainGalleryResponse::Imgs(result)),
    // };
    // Ok(server_package)
    Ok(Some(global::ServerMsg::GalleryMain(result)))
}
