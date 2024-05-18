pub mod con_task;

use std::{io, net::SocketAddr, sync::Arc};

use artcord_mongodb::database::DB;
use artcord_state::{message::prod_client_msg::ClientThresholdMiddleware, misc::throttle_threshold::Threshold};
use chrono::{DateTime, TimeDelta, Utc};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, error, Instrument};

use crate::WsThreshold;

use self::con_task::con_task;

use super::{ws_statistic::WsStatsMsg, ws_throttle::WsThrottle, GetUserAddrMiddleware, WsAppMsg};

pub async fn on_connection(
    con: Result<(TcpStream, SocketAddr), io::Error>,
    throttle: &mut WsThrottle,
    cancellation_token: CancellationToken,
    db: Arc<DB>,
    task_tracker: &TaskTracker,
    ws_addr: &str,
    ws_app_tx: mpsc::Sender<WsAppMsg>,
    admin_ws_stats_tx: mpsc::Sender<WsStatsMsg>,
    threshold: &WsThreshold,
    time: &DateTime<Utc>,
    get_threshold: impl ClientThresholdMiddleware + Send + Sync + Clone + 'static,
    socket_addr_middleware: &(impl GetUserAddrMiddleware + Send + Sync + Clone + 'static),
) {
    let (stream, user_addr) = match con {
        Ok(result) => result,
        Err(err) => {
            debug!("ws({}): error accepting connection: {}", &ws_addr, err);
            return;
        }
    };

    let user_addr = socket_addr_middleware.get_addr(user_addr).await;
    let ip = user_addr.ip();
    
    let reach_max_con = match throttle
        .on_connect(ip, threshold, time, )
        .await
    {
        Ok(max) => max,
        Err(err) => {
            error!("ws({}): failed to run on_connect: {}", &ws_addr, err);
            return;
        }
    };
    if reach_max_con {
        debug!("ws({}): dont connect", &ws_addr);
        return;
    }

    task_tracker.spawn(
        con_task(
            stream,
            cancellation_token,
            db,
            ws_app_tx,
            ip,
            user_addr,
            admin_ws_stats_tx,
            get_threshold,
        )
        .instrument(tracing::trace_span!(
            "ws",
            "{}-{}",
            ws_addr,
            user_addr.to_string()
        )),
    );
}
