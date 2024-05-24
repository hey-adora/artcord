use std::collections::HashMap;
use std::{sync::Arc};

use artcord_leptos_web_sockets::WsRouteKey;
use artcord_mongodb::database::DB;
use artcord_state::{message::prod_server_msg::ServerMsg, model::ws_statistics::TempConIdType};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, trace};
use tokio::sync::{mpsc, oneshot};

use crate::ws::WsAppMsg;
use crate::ws::con::ConMsg;

use super::ResErr;

pub async fn ws_throttle_cached(
    db: Arc<DB>,
    listener_state: bool,
    connection_key: TempConIdType,
    ws_key: WsRouteKey,
    connection_tx: &mpsc::Sender<ConMsg>,
    ws_app_tx: &mpsc::Sender<WsAppMsg>,
) -> Result<Option<ServerMsg>, ResErr> {
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
