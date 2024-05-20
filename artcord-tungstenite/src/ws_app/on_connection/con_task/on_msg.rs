use std::{borrow::BorrowMut, collections::HashMap};

use artcord_state::{message::prod_client_msg::ClientPathType, misc::{throttle_connection::{IpBanReason, LiveThrottleConnectionCount, PathStat}, throttle_threshold::Threshold}, model::ws_statistics::TempConIdType};
use chrono::{DateTime, TimeDelta, Utc};
use futures::{stream::SplitSink, SinkExt};
use tokio::{net::TcpStream, sync::{broadcast, mpsc}};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, trace};
use thiserror::Error;

use crate::ws_app::PathStats;

use super::{ConMsg, GlobalConMsg};

pub async fn on_msg(
    msg: Option<ConMsg>,
    con_id: &TempConIdType,
    client_out: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    global_con_channel_tx: &broadcast::Sender<GlobalConMsg>,
    ip_stats_listeners: &mut HashMap<TempConIdType, mpsc::Sender<ConMsg>>,
    stats: &mut PathStats,
    ban_threshold: &Threshold,
    ban_duration: &TimeDelta,
    banned_until: &mut Option<(DateTime<Utc>, IpBanReason)>,
    time: &DateTime<Utc>,
) -> Result<bool, ReqOnMsgErr> {
    let Some(msg) = msg else {
        trace!("connection msg channel closed");
        return Ok(true);
    };


    match msg {
        ConMsg::Send(msg) => {
            let send_result = client_out.send(msg).await;
            if let Err(err) = send_result {
                debug!("failed to send msg: {}", err);
                return Ok(true);
            }
        }
        ConMsg::Stop => {
            return Ok(true);
        }
        ConMsg::CheckThrottle { path, block_threshold, allow_tx } => {
            let result = stats.inc_path(path, block_threshold, ban_threshold, ban_duration, banned_until, time).await;
            let result = match result {
                artcord_state::misc::throttle_threshold::AllowCon::Allow => {
                    true
                }
                artcord_state::misc::throttle_threshold::AllowCon::AlreadyBanned => {
                    false
                }
                artcord_state::misc::throttle_threshold::AllowCon::Banned(_) => {
                    false
                }
                artcord_state::misc::throttle_threshold::AllowCon::Blocked => {
                    false
                }
                artcord_state::misc::throttle_threshold::AllowCon::Unbanned => {
                    true
                }
            };
            allow_tx.send(result).map_err(|_| ReqOnMsgErr::ThrottleCheckSend)?;
            //let stats = &mut *ip_stats_rx.borrow_mut();
        }
        ConMsg::AddIpStatListener { msg_author, con_tx, current_state_tx } => {
            if *con_id == msg_author {
                return Ok(false);
            }
            current_state_tx.send(stats.paths.clone()).await?;
            ip_stats_listeners.insert(msg_author, con_tx);

        }
    }

    Ok(false)
}

#[derive(Error, Debug)]
pub enum ReqOnMsgErr {
    #[error("failed to send throttle check result.")]
    ThrottleCheckSend,

    #[error("failed to send stats: {0}")]
    SendStatsErr(#[from] mpsc::error::SendError< HashMap<ClientPathType, PathStat> >),

}