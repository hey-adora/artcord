use std::{sync::Arc};

use artcord_mongodb::database::DB;
use artcord_state::message::{
    prod_server_msg::{ServerMsg},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, trace};

use crate::ws_app::{ws_statistic::AdminConStatMsg, ConMsg, WsResError};

pub async fn ws_stats_total_count(
    db: Arc<DB>,
) -> Result<Option<ServerMsg>, WsResError> {
    let amount = db.ws_statistic_total_amount().await?;

    Ok(Some(ServerMsg::WsStatsTotalCount(amount) ))
    // Ok(Some(UserTaskState::Started))
}