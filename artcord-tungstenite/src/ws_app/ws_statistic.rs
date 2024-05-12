use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::{
    message::{
        prod_client_msg::ClientMsgIndexType, prod_perm_key::ProdMsgPermKey, prod_server_msg::ServerMsg
    }, misc::throttle_threshold::Threshold, model::ws_statistics::{TempConIdType, WsStatDb, WsStatTemp, WsStatTempCountItem}
};
use chrono::{DateTime, Utc};
use tokio::{select, sync::{mpsc, oneshot}, task::JoinHandle};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, trace, warn};

use super::{ws_throttle::AdminMsgErr, ConMsg};

pub enum AdminConStatMsg {
    CheckThrottle {
        connection_key: TempConIdType,
        path: ClientMsgIndexType,
        threshold: Threshold,
        result_tx: oneshot::Sender<bool>,
    },

    Inc {
        connection_key: TempConIdType,
        path: ClientMsgIndexType,
    },

    AddTrack {
        connection_key: TempConIdType,
        tx: mpsc::Sender<ConMsg>,
        ip: String,
        addr: String,
        // ws_key: WsRouteKey,
    },

    AddListener {
        connection_key: TempConIdType,
        tx: mpsc::Sender<ConMsg>,
        addr: String,
        ws_key: WsRouteKey,
    },

    RemoveListener {
        connection_key: TempConIdType,
    },

    StopTrack {
        connection_key: TempConIdType,
    },
}

pub async fn create_admin_con_stat_task(
    task_tracker: &TaskTracker,
    cancelation_token: &CancellationToken,
    db: Arc<DB>,
) -> (JoinHandle<()>, mpsc::Sender<AdminConStatMsg>) {
    let (tx, rx) = mpsc::channel::<AdminConStatMsg>(100);
    let listener_task = task_tracker.spawn(admin_stat_task(rx, cancelation_token.clone(), db));
    (listener_task, tx)
}

pub async fn admin_stat_task(
    mut rx: mpsc::Receiver<AdminConStatMsg>,
    cancelation_token: CancellationToken,
    db: Arc<DB>,
) {
    let mut listener_list: HashMap<TempConIdType, (WsRouteKey, mpsc::Sender<ConMsg>)> = HashMap::new();
    let mut con_list: HashMap<TempConIdType, mpsc::Sender<ConMsg>> = HashMap::new();
    let mut stats: HashMap<TempConIdType, WsStatTemp> = HashMap::new();

    loop {
        select! {
            msg = rx.recv() => {
                let exit = on_msg(msg, &mut listener_list, &mut stats, &mut con_list, &*db).await;
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

    let unsaved_stats = match WsStatDb::from_hashmap_temp_stats(stats) {
        Ok(stats) => stats,
        Err(err) => {
            error!("ws_stats: error converting from WsStatsTemp to WsStats {}", err);
            return;
        }
    };
    let result = db.ws_statistic_insert_many(unsaved_stats).await;
    if let Err(err) = result {
        error!("ws_stats: saving unsaved_stats {}", err);
    }
}

pub async fn on_msg(
    msg: Option<AdminConStatMsg>,
    list: &mut HashMap<TempConIdType, (WsRouteKey, mpsc::Sender<ConMsg>)>,
    stats: &mut HashMap<TempConIdType, WsStatTemp>,
    cons: &mut HashMap<TempConIdType, mpsc::Sender<ConMsg>>,
    db: &DB,
) -> Result<bool, AdminMsgErr> {
    let Some(msg) = msg else {
        return Ok(true);
    };

    match msg {
        AdminConStatMsg::CheckThrottle { connection_key, path, threshold, result_tx } => {
            result_tx.send(true);
        }
        AdminConStatMsg::Inc {
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
            let count = stat.count.entry(path).or_insert(WsStatTempCountItem::default());
            count.total_count += 1;
            count.count += 1;

            let update_msg = ServerMsg::WsLiveStatsUpdateInc {
                con_key: connection_key,
                path,
            };

            for (con_key, (ws_key, tx)) in list {
                let update_msg: WsPackage<ServerMsg> = (ws_key.clone(), update_msg.clone());
                let update_msg = ServerMsg::as_bytes(update_msg)?;
                let update_msg = Message::binary(update_msg);
                tx.send(ConMsg::Send(update_msg.clone())).await?;
            }
        }
        AdminConStatMsg::AddTrack {
            connection_key,
            tx,
            ip,
            addr,
            // ws_key,
        } => {
            // let statistics = vec![Statistic::new(addr.clone().to_string())];

            trace!("admin stats: added to track: {}", &connection_key);
            let current_con_stats: WsStatTemp = stats
                .entry(connection_key.clone())
                .or_insert(WsStatTemp::new(ip, addr.clone(), Utc::now().timestamp_millis()))
                .clone();
            cons.insert(connection_key.clone(), tx);

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
            for (con_key, (ws_key, tx)) in list {
                let update_msg: WsPackage<ServerMsg> = (ws_key.clone(), update_msg.clone());
                let update_msg = ServerMsg::as_bytes(update_msg)?;
                let update_msg = Message::binary(update_msg);
                tx.send(ConMsg::Send(update_msg.clone())).await?;
            }
        }
        AdminConStatMsg::AddListener {
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
        AdminConStatMsg::RemoveListener { connection_key } => {
            trace!("admin stats: removed from recv: {}", &connection_key);
            list.remove(&connection_key);
        }
        AdminConStatMsg::StopTrack { connection_key } => {
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

            let stat_db: WsStatDb = match WsStatDb::from_temp(stat.clone(), uuid::Uuid::from_u128(connection_key).to_string(), Utc::now().timestamp_millis()) {
                Ok(stat) => stat,
                Err(err) => {
                    warn!("admin stats: missing connection entry: {}", &connection_key);
                    return Ok(false);
                }
            };

            db.ws_statistic_insert_one(stat_db).await?;
            stats.remove(&connection_key);
            cons.remove(&connection_key);
            list.remove(&connection_key);

            for (con_key, (ws_key, tx)) in list {
                let update_msg: WsPackage<ServerMsg> = (ws_key.clone(), update_msg.clone());
                let update_msg = ServerMsg::as_bytes(update_msg)?;
                let update_msg = Message::binary(update_msg);
                tx.send(ConMsg::Send(update_msg.clone())).await?;
            }
        }
    }

    Ok(false)
}
