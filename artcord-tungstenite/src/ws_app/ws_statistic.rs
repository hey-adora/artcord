use std::{collections::HashMap, net::{IpAddr, SocketAddr}, sync::Arc};

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::{
    message::{
        prod_client_msg::ClientPathType, prod_perm_key::ProdMsgPermKey, prod_server_msg::ServerMsg
    }, misc::{throttle_connection::IpBanReason, throttle_threshold::{AllowCon, Threshold}}, model::ws_statistics::{DbWsStat, TempConIdType, WsStat, WsStatPath}, util::time::TimeMiddleware
};
use chrono::{DateTime, TimeDelta, Utc};
use tokio::{select, sync::{mpsc, oneshot}, task::JoinHandle};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, trace, warn};

// use crate::{WS_BRUTE_BAN_THRESHOLD, WS_BRUTE_BLOCK_THRESHOLD, WS_BRUTE_THRESHOLD_BAN_DURATION};

use crate::WsThreshold;

use super::{ws_throttle::WsStatsOnMsgErr, ConMsg, WsAppMsg};

pub enum WsStatsMsg {
    CheckThrottle {
        connection_key: TempConIdType,
        path: ClientPathType,
        threshold: Threshold,
        result_tx: oneshot::Sender<bool>,
    },

    Inc {
        connection_key: TempConIdType,
        path: ClientPathType,
    },

    AddTrack {
        connection_key: TempConIdType,
        tx: mpsc::Sender<ConMsg>,
        ip: IpAddr,
        addr: SocketAddr,
        // ws_key: WsRouteKey,
    },

    AddListener {
        connection_key: TempConIdType,
        tx: mpsc::Sender<ConMsg>,
        addr: SocketAddr,
        ws_key: WsRouteKey,
    },

    RemoveListener {
        connection_key: TempConIdType,
    },

    StopTrack {
        connection_key: TempConIdType,
    },
}

pub async fn create_stat_task(
    ws_app_tx: mpsc::Sender<WsAppMsg>,
    task_tracker: &TaskTracker,
    cancelation_token: &CancellationToken,
    threshold: &WsThreshold,
    db: Arc<DB>,
    time_machine: impl TimeMiddleware + Send + 'static,
) -> (JoinHandle<()>, mpsc::Sender<WsStatsMsg>) {
    let (tx, rx) = mpsc::channel::<WsStatsMsg>(100);
    let listener_task = task_tracker.spawn(stat_task(ws_app_tx, rx, cancelation_token.clone(), threshold.clone(), db, time_machine));
    (listener_task, tx)
}

pub async fn stat_task(
    ws_app_tx: mpsc::Sender<WsAppMsg>,
    mut rx: mpsc::Receiver<WsStatsMsg>,
    cancelation_token: CancellationToken,
    threshold: WsThreshold,
    db: Arc<DB>,
    time_machine: impl TimeMiddleware,
) {
    let mut listener_list: HashMap<TempConIdType, (WsRouteKey, mpsc::Sender<ConMsg>)> = HashMap::new();
    //let mut con_list: HashMap<TempConIdType, mpsc::Sender<ConMsg>> = HashMap::new();
    let mut stats: HashMap<TempConIdType, WsStat> = HashMap::new();

    
    loop {
        select! {
            msg = rx.recv() => {
                let exit = on_msg(&ws_app_tx, msg, &mut listener_list, &mut stats, &*db, &threshold.ws_stat_threshold, threshold.ws_stat_ban_duration, time_machine.get_time().await).await;
                match exit {
                    Ok(exit) => {
                        if exit {
                            break;
                        }
                    }
                    Err(err) => {
                        error!("throttle listener error: {}", err);
                    }
                }
            },
            _ = cancelation_token.cancelled() => {
                trace!("throttle listener_task cancelled");
                break;
            },
        }
    }

    let unsaved_stats = match DbWsStat::from_hashmap_ws_stats(stats, time_machine.get_time().await) {
        Ok(stats) => stats,
        Err(err) => {
            error!("ws_stats: error converting from WsStatsTemp to WsStats {}", err);
            return;
        }
    };
    if !unsaved_stats.is_empty() {
        trace!("ws_stats: saving stats: {:#?}", unsaved_stats);
        let result = db.ws_statistic_insert_many(unsaved_stats).await;
        if let Err(err) = result {
            error!("ws_stats: saving unsaved_stats {}", err);
        }
    } else {
        trace!("ws_stats: no stats to save");
    }
}

pub async fn on_msg(
    ws_app_tx: &mpsc::Sender<WsAppMsg>,
    msg: Option<WsStatsMsg>,
    list: &mut HashMap<TempConIdType, (WsRouteKey, mpsc::Sender<ConMsg>)>,
    stats: &mut HashMap<TempConIdType, WsStat>,
    //cons: &mut HashMap<TempConIdType, mpsc::Sender<ConMsg>>,
    db: &DB,
    ban_threshold: &Threshold,
    ban_duration: TimeDelta,
    time: DateTime<Utc>,
) -> Result<bool, WsStatsOnMsgErr> {
    let Some(msg) = msg else {
        return Ok(true);
    };

    match msg {
        WsStatsMsg::CheckThrottle { connection_key, path, threshold, result_tx } => {
            let stat = stats.get_mut(&connection_key);
            let Some(stat) = stat else {
                warn!("admin stats: missing connection entry: {}", &connection_key);
                // stats.insert(connection_key, AdminStat::new(kj, is_connected, count))
                return Ok(false);
            };

           

            let path = stat.count.entry(path).or_insert_with(|| WsStatPath::new(time));

            let result = path.throttle.allow(&threshold, ban_threshold, IpBanReason::WsRouteBruteForceDetected, ban_duration, &time);

            let result = match result {
                AllowCon::Allow | AllowCon::Unbanned => true,
                AllowCon::Banned((until, reason)) => {
                    ws_app_tx.send(WsAppMsg::Ban { ip: stat.ip, until, reason }).await?;
                    // if let Err(err) = send_result {
                    //     error!("throttle: error sending ban even to ws_app: {}", err);
                    // }
                    false
                }
                _ => false
            };

            trace!("stats: CheckThrottle: {:#?}", &stat);

            result_tx.send(result).map_err(|_|WsStatsOnMsgErr::SendCheckThrottle)?;
            //stats.
        }
        WsStatsMsg::Inc {
            connection_key,
            path,
        } => {
            // let entry = stats.entry(connection_key).or_insert(connection_key);
            let stat = stats.get_mut(&connection_key);
            let Some(stat) = stat else {
                warn!("admin stats: missing connection entry: {}", &connection_key);
                // stats.insert(connection_key, AdminStat::new(kj, is_connected, count))
                return Ok(false);
            };
            let count = stat.count.entry(path).or_insert_with(|| WsStatPath::new(time));
            count.total_count += 1;
            count.count += 1;

            let update_msg = ServerMsg::WsLiveStatsUpdateInc {
                con_key: connection_key,
                path,
            };
            send_to_listeners(list, update_msg).await?;
        }
        WsStatsMsg::AddTrack {
            connection_key,
            tx,
            ip,
            addr,
            // ws_key,
        } => {
            // let statistics = vec![Statistic::new(addr.clone().to_string())];

            trace!("admin stats: added to track: {}", &connection_key);
            let current_con_stats: WsStat = stats
                .entry(connection_key.clone())
                .or_insert(WsStat::new(ip, addr.clone(), time))
                .clone();
            //cons.insert(connection_key.clone(), tx);

            // let msg = ServerMsg::AdminStats(AdminStatsRes::Started(stats.clone()));
            // let msg: WsPackage<ServerMsg> = (ws_key.clone(), msg);
            // trace!("admin stats: sending: {:?}", &msg);
            // let msg = ServerMsg::as_bytes(msg)?;
            // let msg = Message::binary(msg);
            // tx.send(ConMsg::Send(msg)).await?;
            // list.insert(connection_key.clone(), (ws_key, tx));

            let update_msg = ServerMsg::WsLiveStatsUpdateAddedStat {
                con_key: connection_key,
                stat: current_con_stats,
            };
            send_to_listeners(list, update_msg).await?;
        }
        WsStatsMsg::AddListener {
            connection_key,
            tx,
            addr,
            ws_key,
        } => {
            // let statistics = vec![Statistic::new(addr.clone().to_string())];

            // let current_con_stats = stats.get(&connection_key);
            // // .entry(connection_key.clone())
            // // .or_insert(AdminStat::new(addr.clone(), false, HashMap::new()))
            // // .clone();
            //
            // let Some(current_con_stats) = current_con_stats else {
            //     return Ok(false);
            // };

            let msg = ServerMsg::WsLiveStatsStarted(stats.clone());
            let msg: WsPackage<ServerMsg> = (ws_key.clone(), msg);
            trace!("admin stats: sending: {:?}", &msg);
            let msg = ServerMsg::as_bytes(msg)?;
            let msg = Message::binary(msg);
            tx.send(ConMsg::Send(msg)).await?;

            // let update_msg = ServerMsg::AdminStats(AdminStatsRes::UpdateAddedNew {
            //     con_key: connection_key,
            //     stat: current_con_stats,
            // });
            // for (con_key, (ws_key, tx)) in list {
            //     let update_msg: WsPackage<ServerMsg> = (ws_key.clone(), update_msg.clone());
            //     let update_msg = ServerMsg::as_bytes(update_msg)?;
            //     let update_msg = Message::binary(update_msg);
            //     tx.send(ConMsg::Send(update_msg.clone())).await;
            // }

            list.insert(connection_key.clone(), (ws_key, tx));

          
        }
        WsStatsMsg::RemoveListener { connection_key } => {
            trace!("admin stats: removed from recv: {}", &connection_key);
            list.remove(&connection_key);
        }
        WsStatsMsg::StopTrack { connection_key } => {
            trace!("admin stats: removed from track: {}", &connection_key);
            let stat = stats.get(&connection_key);
            let Some(stat) = stat else {
                warn!("admin stats: missing connection entry: {}", &connection_key);
                // stats.insert(connection_key, AdminStat::new(kj, is_connected, count))
                return Ok(false);
            };

            let update_msg = ServerMsg::WsLiveStatsUpdateRemoveStat {
                con_key: connection_key.clone(),
            };

            let disconnected_at = time;
            let stat_db: DbWsStat = match DbWsStat::from_ws_stat(stat.clone(), uuid::Uuid::from_u128(connection_key).to_string(), disconnected_at, disconnected_at) {
                Ok(stat) => stat,
                Err(err) => {
                    warn!("admin stats: missing connection entry: {}", &connection_key);
                    return Ok(false);
                }
            };

            db.ws_statistic_insert_one(stat_db).await?;
            stats.remove(&connection_key);
            //cons.remove(&connection_key);
            list.remove(&connection_key);

            send_to_listeners(list, update_msg).await?;
        }
    }

    Ok(false)
}

pub async fn send_to_listeners(list: &mut HashMap<TempConIdType, (WsRouteKey, mpsc::Sender<ConMsg>)>, update_msg: ServerMsg) -> Result<(), WsStatsOnMsgErr> {
    for (con_key, (ws_key, tx)) in list {
        let update_msg: WsPackage<ServerMsg> = (ws_key.clone(), update_msg.clone());
        let update_msg = ServerMsg::as_bytes(update_msg)?;
        let update_msg = Message::binary(update_msg);
        tx.send(ConMsg::Send(update_msg.clone())).await?;
    }
    Ok(())
}