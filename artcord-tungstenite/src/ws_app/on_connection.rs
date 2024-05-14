pub mod con_task;

use std::{io, net::SocketAddr, sync::Arc};

use artcord_mongodb::database::DB;
use artcord_state::misc::throttle_threshold::Threshold;
use chrono::{DateTime, TimeDelta, Utc};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, error, Instrument};

use crate::WsThreshold;

use self::con_task::con_task;

use super::{ws_statistic::WsStatsMsg, ws_throttle::WsThrottle, WsAppMsg};

pub async fn on_connection(
    con: Result<(TcpStream, SocketAddr), io::Error>,
    throttle: &mut WsThrottle,
    cancellation_token: &CancellationToken,
    db: &Arc<DB>,
    task_tracker: &TaskTracker,
    ws_addr: &str,
    ws_app_tx: &mpsc::Sender<WsAppMsg>,
    admin_ws_stats_tx: &mpsc::Sender<WsStatsMsg>,
    threshold: &WsThreshold,
    time: DateTime<Utc>,
) {
    let (stream, user_addr) = match con {
        Ok(result) => result,
        Err(err) => {
            debug!("ws({}): error accepting connection: {}", &ws_addr, err);
            return;
        }
    };

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
