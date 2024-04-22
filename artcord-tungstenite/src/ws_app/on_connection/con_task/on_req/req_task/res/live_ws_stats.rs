use std::{net::SocketAddr, sync::Arc, time::Duration};

use artcord_leptos_web_sockets::{WsError, WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::{
    prod_perm_key::ProdMsgPermKey,
    prod_server_msg::{LiveWsStatsRes, ServerMsg},
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

use crate::ws_app::{ws_statistic::AdminConStatMsg, ConMsg, WsResError};

pub async fn live_ws_stats(
    db: Arc<DB>,
    listener_state: bool,
    connection_key: String,
    ws_key: WsRouteKey,
    addr: SocketAddr,
    connection_tx: &mpsc::Sender<ConMsg>,
    admin_ws_stats_tx: mpsc::Sender<AdminConStatMsg>,
    // mut admin_task: UserTask,

    // task_tracker: TaskTracker,
    // is_admin_throttle_listener_active: Arc<Mutex<Option<JoinHandle<()>>>>,
    // mut cancel_recv: broadcast::Receiver<bool>,
    // cancel_send: broadcast::Sender<bool>,
    // admin_throttle_listener_recv_close: oneshot::Receiver<bool>,
) -> Result<Option<ServerMsg>, WsResError> {
    if listener_state {
        admin_ws_stats_tx
            .send(AdminConStatMsg::AddRecv {
                connection_key,
                tx: connection_tx.clone(),
                addr: addr.to_string(),
                ws_key,
            })
            .await?;
    } else {
        admin_ws_stats_tx
            .send(AdminConStatMsg::RemoveRecv { connection_key })
            .await?;
    }
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
