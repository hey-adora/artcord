use std::{net::SocketAddr, sync::Arc, time::Duration};

use artcord_leptos_web_sockets::{WsError, WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::{message::{
    prod_perm_key::ProdMsgPermKey,
    prod_server_msg::ServerMsg,
}, model::ws_statistics::TempConIdType};
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
    connection_key: TempConIdType,
    ws_key: WsRouteKey,
    addr: SocketAddr,
    connection_tx: &mpsc::Sender<ConMsg>,
    admin_ws_stats_tx: mpsc::Sender<AdminConStatMsg>,

) -> Result<Option<ServerMsg>, WsResError> {
    if listener_state {
        admin_ws_stats_tx
            .send(AdminConStatMsg::AddListener {
                connection_key,
                tx: connection_tx.clone(),
                addr: addr.to_string(),
                ws_key,
            })
            .await?;
    } else {
        admin_ws_stats_tx
            .send(AdminConStatMsg::RemoveListener { connection_key })
            .await?;
    }

    Ok(None)
}
