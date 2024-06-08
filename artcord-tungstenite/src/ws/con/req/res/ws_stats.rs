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

pub async fn live(
    listener_state: bool,
    connection_tx: &mpsc::Sender<ConMsg>,
    res_key: WsRouteKey,
) -> Result<Option<global::ServerMsg>, ResErr> {
    //let (tx, rx) = oneshot::channel();
    if listener_state {
        connection_tx
        .send(ConMsg::AddWsThrottleListener { res_key })
        .await?;
    } else {
        connection_tx
        .send(ConMsg::RemoveWsThrottleListener)
        .await?;
    }
    Ok(None)
}

pub async fn paged(
    db: Arc<DB>,
    page: u64,
    amount: u64,
    from: i64,
) -> Result<Option<global::ServerMsg>, ResErr> {
    let ws_cons = db.ws_statistic_paged_latest(page, amount, from).await?;
    let saved_ws_cons = ws_cons.into_iter().map(|con| con.try_into()).collect::<Result<Vec<global::SavedWsCon>, _>>()?;

    Ok(Some(global::ServerMsg::WsSavedStatsPage(saved_ws_cons)))
}

pub async fn graph(
    db: Arc<DB>,
    from: i64,
    to: i64,
    unique_ip: bool,
) -> Result<Option<global::ServerMsg>, ResErr> {
    let graph_data = db.ws_stats_graph(from, to, unique_ip).await?;

    Ok(Some(global::ServerMsg::WsSavedStatsGraph(graph_data)))
}

pub async fn total_count(
    db: Arc<DB>,
    from: Option<i64>,
) -> Result<Option<global::ServerMsg>, ResErr> {
    let amount = db.ws_statistic_total_amount(from).await?;

    //Ok(Some(ServerMsg::WsStatsTotalCount(amount) ))
    Ok(None)
}

pub async fn pagination(
    db: Arc<DB>,
    page: u64,
    amount: u64,
) -> Result<Option<global::ServerMsg>, ResErr> {
    let (total_count, latest, stats) = db.ws_statistic_with_pagination_latest(page, amount).await?;
    let saved_ws_cons = stats.into_iter().map(|img| img.try_into()).collect::<Result<Vec<global::SavedWsCon>, _>>()?;

    Ok(Some(global::ServerMsg::WsSavedStatsWithPagination { total_count, latest, stats: saved_ws_cons } ))
}

