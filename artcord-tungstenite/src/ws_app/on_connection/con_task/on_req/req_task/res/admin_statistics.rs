use std::{net::SocketAddr, sync::Arc, time::Duration};

use artcord_leptos_web_sockets::{WsError, WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::{
    prod_perm_key::ProdMsgPermKey,
    prod_server_msg::{AdminStatsRes, ServerMsg},
};
use futures::channel::oneshot::Cancellation;
use thiserror::Error;
use tokio::{
    select,
    sync::{broadcast, mpsc, oneshot, Mutex, RwLock},
    task::{JoinError, JoinHandle},
    time::{self, sleep},
};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, trace};

use crate::ws_app::{ws_statistic::WsThrottleListenerMsg, ConMsg, WsResError};

pub async fn ws_hadnle_admin_throttle(
    db: Arc<DB>,
    listener_state: bool,
    connection_key: uuid::Uuid,
    ws_key: WsRouteKey<u128, ProdMsgPermKey>,
    addr: SocketAddr,
    connection_tx: &mpsc::Sender<ConMsg>,
    throttle_tx: mpsc::Sender<WsThrottleListenerMsg>,
    // mut admin_task: UserTask,

    // task_tracker: TaskTracker,
    // is_admin_throttle_listener_active: Arc<Mutex<Option<JoinHandle<()>>>>,
    // mut cancel_recv: broadcast::Receiver<bool>,
    // cancel_send: broadcast::Sender<bool>,
    // admin_throttle_listener_recv_close: oneshot::Receiver<bool>,
) -> Result<Option<ServerMsg>, WsResError> {
    throttle_tx
        .send(WsThrottleListenerMsg::Add {
            connection_key,
            tx: connection_tx.clone(),
            addr,
            ws_key,
        })
        .await?;
    // let result = db.user_find_one(&user_id).await?;
    //
    // let Some(result) = result else {
    //     return Ok(artcord_state::message::prod_server_msg::UserResponse::UserNotFound);
    // };
    //Ok(artcord_state::message::prod_server_msg::UserTaskState::AlreadyStopped)
    // admin_task
    //     .set_output_task(move |_| {
    //         Box::pin(async move {
    //             debug!("YO YO YO MF ");
    //         })
    //     })
    //     .await;

    Ok(None)
    // Ok(Some(UserTaskState::Started))
}

// #[derive(Error, Debug)]
// pub enum WsHandleAdminThrottleError {
//     #[error("Mongodb error: {0}")]
//     MongoDB(#[from] mongodb::error::Error),
//
//     #[error("Tokio JoinError error: {0}")]
//     JoinError(#[from] JoinError),
// }
