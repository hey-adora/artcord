use std::collections::HashMap;
use std::{sync::Arc};

use artcord_leptos_web_sockets::WsRouteKey;
use artcord_mongodb::database::DB;
use artcord_state::global;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, trace};
use tokio::sync::{mpsc, oneshot};

use crate::ws::WsAppMsg;
use crate::ws::con::ConMsg;

use super::ResErr;

pub async fn ws_throttle_cached(
    db: Arc<DB>,
    listener_state: bool,
    connection_key: global::TempConIdType,
    ws_key: WsRouteKey,
    connection_tx: &mpsc::Sender<ConMsg>,
    ws_app_tx: &mpsc::Sender<WsAppMsg>,
) -> Result<Option<global::ServerMsg>, ResErr> {
    // if listener_state {
    //     //let live_stats = HashMap::new();
    //     let (tx, rx) = oneshot::channel();
    //     connection_tx
    //         .send(ConMsg::AddWsThrottleListener { current_state_tx: tx })
    //         .await?;
    //     return ServerMsg::Live
    // } else {
    //     connection_tx
    //         .send(WsAppMsg::RemoveListener {
    //             connection_key,
    //             tx: connection_tx.clone(),
    //             ws_key,
    //         })
    //         .await?;
    // }

    Ok(None)
}
