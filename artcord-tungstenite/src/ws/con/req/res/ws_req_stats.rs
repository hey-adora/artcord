use std::{sync::Arc};

use artcord_leptos_web_sockets::WsRouteKey;
use artcord_mongodb::database::DB;
use artcord_state::{message::prod_server_msg::ServerMsg, model::ws_statistics::TempConIdType};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, trace};
use tokio::sync::mpsc;

use crate::ws::WsAppMsg;
use crate::ws::con::ConMsg;

use super::ResErr;


pub async fn paged(
    db: Arc<DB>,
    page: u64,
    amount: u64,
    from: i64,
) -> Result<Option<ServerMsg>, ResErr> {
    let imgs = db.ws_statistic_paged_latest(page, amount, from).await?;

    Ok(Some(ServerMsg::WsStatsPage(imgs)))
}

pub async fn ranged(
    db: Arc<DB>,
    from: i64,
    to: i64,
    unique_ip: bool,
) -> Result<Option<ServerMsg>, ResErr> {
    let imgs = db.ws_statistic_ranged_latest(from, to, unique_ip).await?;

    Ok(Some(ServerMsg::WsStatsGraph(imgs)))
}

pub async fn total_count(
    db: Arc<DB>,
    from: Option<i64>,
) -> Result<Option<ServerMsg>, ResErr> {
    let amount = db.ws_statistic_total_amount(from).await?;

    Ok(Some(ServerMsg::WsStatsTotalCount(amount) ))
}

pub async fn pagination(
    db: Arc<DB>,
    page: u64,
    amount: u64,
) -> Result<Option<ServerMsg>, ResErr> {
    let (total_count, latest, stats) = db.ws_statistic_with_pagination_latest(page, amount).await?;

    Ok(Some(ServerMsg::WsStatsWithPagination { total_count, latest, stats } ))
}
