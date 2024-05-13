use std::{net::SocketAddr, sync::Arc, time::Duration};

use artcord_leptos_web_sockets::{WsError, WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::{
    message::{prod_perm_key::ProdMsgPermKey, prod_server_msg::ServerMsg},
    model::ws_statistics::TempConIdType,
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

use crate::ws_app::{ws_statistic::WsStatsMsg, ConMsg, WsAppMsg, WsResError};

pub async fn ws_throttle_cached(
    db: Arc<DB>,
    listener_state: bool,
    connection_key: TempConIdType,
    ws_key: WsRouteKey,
    connection_tx: &mpsc::Sender<ConMsg>,
    ws_app_tx: &mpsc::Sender<WsAppMsg>,
) -> Result<Option<ServerMsg>, WsResError> {
    if listener_state {
        ws_app_tx
            .send(WsAppMsg::AddListener {
                connection_key,
                tx: connection_tx.clone(),
                ws_key,
            })
            .await?;
    } else {
        ws_app_tx
            .send(WsAppMsg::RemoveListener {
                connection_key,
                tx: connection_tx.clone(),
                ws_key,
            })
            .await?;
    }

    Ok(None)
}
