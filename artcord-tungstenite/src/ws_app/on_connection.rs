pub mod con_task;

use std::{io, net::SocketAddr, sync::Arc};

use artcord_mongodb::database::DB;
use artcord_state::{
    message::prod_client_msg::ClientThresholdMiddleware, misc::throttle_threshold::Threshold, util::time::TimeMiddleware,
};
use chrono::{DateTime, TimeDelta, Utc};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, error, Instrument};

use crate::WsThreshold;

// use self::con_task::con_task;

use self::con_task::Con;

use super::{
    ws_statistic::WsStatsMsg, ws_throttle::WsThrottle, GetUserAddrMiddleware, GlobalConChannel,
    WsAppMsg,
};

pub async fn on_connection(
    con: Result<(TcpStream, SocketAddr), io::Error>,
    throttle: &mut WsThrottle,
    cancellation_token: CancellationToken,
    db: Arc<DB>,
    task_tracker: &TaskTracker,
    ws_addr: &str,
    ws_app_tx: mpsc::Sender<WsAppMsg>,
    admin_ws_stats_tx: mpsc::Sender<WsStatsMsg>,
    ws_threshold: &WsThreshold,
    get_threshold: impl ClientThresholdMiddleware + Send + Sync + Clone + 'static,
    socket_addr_middleware: &(impl GetUserAddrMiddleware + Send + Sync + Clone + 'static),
    time_middleware: impl TimeMiddleware + Clone + Sync + Send + 'static,
    (global_con_channel_tx, global_con_channel_rx): &GlobalConChannel,
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

    let time = time_middleware.get_time().await;

    let allow = match throttle.on_connect(ip, ws_threshold, &time).await {
        Ok(max) => max,
        Err(err) => {
            error!("ws({}): failed to run on_connect: {}", &ws_addr, err);
            return;
        }
    };
    if !allow {
        debug!("ws({}): dont connect", &ws_addr);
        return;
    };

    task_tracker.spawn(
        Con::connect(
            stream,
            cancellation_token,
            db,
            ws_app_tx,
            ip,
            user_addr,
            (
                global_con_channel_tx.clone(),
                global_con_channel_tx.subscribe(),
            ),
            ws_threshold.ws_stat_threshold.clone(),
            ws_threshold.ws_stat_ban_duration.clone(),
            time_middleware,
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
