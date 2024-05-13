use std::{sync::Arc};

use artcord_mongodb::database::DB;
use artcord_state::message::{
    prod_server_msg::{ServerMsg},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, trace};

use crate::ws_app::{ws_statistic::WsStatsMsg, ConMsg, WsResError};

pub async fn ws_stats_with_pagination(
    db: Arc<DB>,
    page: u64,
    amount: u64,
) -> Result<Option<ServerMsg>, WsResError> {
    let (total_count, latest, stats) = db.ws_statistic_with_pagination_latest(page, amount).await?;

    Ok(Some(ServerMsg::WsStatsWithPagination { total_count, latest, stats } ))
}
