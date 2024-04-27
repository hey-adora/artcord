use std::{sync::Arc};

use artcord_mongodb::database::DB;
use artcord_state::message::{
    prod_server_msg::{ServerMsg},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, trace};

use crate::ws_app::{ws_statistic::AdminConStatMsg, ConMsg, WsResError};

pub async fn ws_stats_paged(
    db: Arc<DB>,
    page: u64,
    amount: u64,
) -> Result<Option<ServerMsg>, WsResError> {
    let imgs = db.ws_statistic_paged_latest(page, amount).await?;

    Ok(Some(ServerMsg::WsStatsPage(imgs)))
}