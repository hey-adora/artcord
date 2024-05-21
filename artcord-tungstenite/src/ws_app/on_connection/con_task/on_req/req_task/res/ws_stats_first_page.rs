use std::{sync::Arc};

use artcord_mongodb::database::DB;
use artcord_state::message::{
    prod_server_msg::{ServerMsg},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, trace};

use crate::ws::{ws_statistic::WsStatsMsg, ConMsg, WsResError};

// pub async fn ws_stats_first_page(
//     db: Arc<DB>,
//     amount: u64,
// ) -> Result<Option<ServerMsg>, WsResError> {
//     let total_amount = db.ws_statistic_total_amount(None).await?;
//     let imgs = db.ws_statistic_paged_latest(0, amount, None).await?;

//     Ok(Some(ServerMsg::WsStatsFirstPage { total_count: total_amount, first_page: imgs }))
// }
