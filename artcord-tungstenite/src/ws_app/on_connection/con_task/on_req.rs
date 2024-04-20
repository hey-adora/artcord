use std::net::SocketAddr;
use std::sync::Arc;

use artcord_mongodb::database::DB;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::task::TaskTracker;
use tracing::{debug, trace};

use crate::ws_app::ws_statistic::AdminConStatMsg;

use self::req_task::req_task;

use super::ConMsg;

pub mod req_task;

pub async fn on_req(
    result: Option<Result<Message, tokio_tungstenite::tungstenite::error::Error>>,
    // mut client_in: SplitStream<WebSocketStream<TcpStream>>,
    user_task_tracker: &TaskTracker,
    db: &Arc<DB>,
    connection_task_tx: &mpsc::Sender<ConMsg>,
    admin_ws_stats_tx: &mpsc::Sender<AdminConStatMsg>,
    connection_key: &String,
    addr: &SocketAddr,
) -> bool {
    let Some(result) = result else {
        trace!("read.next() returned None");
        return true;
    };

    let client_msg = match result {
        Ok(result) => result,
        Err(err) => {
            debug!("recv msg error: {}", err);
            return false;
        }
    };

    user_task_tracker.spawn(req_task(
        client_msg,
        db.clone(),
        connection_task_tx.clone(),
        admin_ws_stats_tx.clone(),
        connection_key.clone(),
        addr.clone(),
    ));

    false
}
