pub mod con_task;

use std::{io, net::SocketAddr, sync::Arc};

use artcord_mongodb::database::DB;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, Instrument};

use self::con_task::con_task;

use super::{ws_statistic::AdminConStatMsg, ws_throttle::WsThrottle, WsAppMsg};

pub async fn on_connection(
    con: Result<(TcpStream, SocketAddr), io::Error>,
    throttle: &mut WsThrottle,
    cancellation_token: &CancellationToken,
    db: &Arc<DB>,
    task_tracker: &TaskTracker,
    ws_addr: &str,
    ws_app_tx: &mpsc::Sender<WsAppMsg>,
    admin_ws_stats_tx: &mpsc::Sender<AdminConStatMsg>,
) {
    let (stream, user_addr) = match con {
        Ok(result) => result,
        Err(err) => {
            debug!("ws({}): error accepting connection: {}", &ws_addr, err);
            return;
        }
    };

    let ip = user_addr.ip();
    if throttle.is_bad(ip).await {
        debug!("ws({}): dont connect", &ws_addr);
        return;
    }

    task_tracker.spawn(
        con_task(
            stream,
            cancellation_token.clone(),
            db.clone(),
            ws_app_tx.clone(),
            ip,
            user_addr,
            admin_ws_stats_tx.clone(),
        )
        .instrument(tracing::trace_span!(
            "ws",
            "{}-{}",
            ws_addr,
            user_addr.to_string()
        )),
    );
}
