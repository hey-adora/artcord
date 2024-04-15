use std::{collections::HashMap, net::SocketAddr};

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_state::{
    message::{
        prod_client_msg::WsPath,
        prod_perm_key::ProdMsgPermKey,
        prod_server_msg::{AdminStat, AdminStatsRes, ServerMsg},
    },
    model::statistics::Statistic,
};
use tokio::{select, sync::mpsc, task::JoinHandle};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, trace};

use super::{ws_throttle::AdminMsgErr, ConMsg};

pub enum AdminConStatMsg {
    Add {
        connection_key: String,
        tx: mpsc::Sender<ConMsg>,
        addr: String,
        ws_key: WsRouteKey,
    },

    Remove {
        key: String,
    },
}

pub async fn create_admin_con_stat_task(
    task_tracker: &TaskTracker,
    cancelation_token: &CancellationToken,
) -> (JoinHandle<()>, mpsc::Sender<AdminConStatMsg>) {
    let (tx, rx) = mpsc::channel::<AdminConStatMsg>(100);
    let listener_task = task_tracker.spawn(admin_stat_task(rx, cancelation_token.clone()));
    (listener_task, tx)
}

pub async fn admin_stat_task(
    mut rx: mpsc::Receiver<AdminConStatMsg>,
    cancelation_token: CancellationToken,
) {
    let mut listener_list: HashMap<String, (WsRouteKey, mpsc::Sender<ConMsg>)> = HashMap::new();
    let mut stats: HashMap<String, AdminStat> = HashMap::new();

    loop {
        select! {
            msg = rx.recv() => {
                let exit = on_msg(msg, &mut listener_list, &mut stats).await;
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
}

pub async fn on_msg(
    msg: Option<AdminConStatMsg>,
    list: &mut HashMap<String, (WsRouteKey, mpsc::Sender<ConMsg>)>,
    stats: &mut HashMap<String, AdminStat>,
) -> Result<bool, AdminMsgErr> {
    let Some(msg) = msg else {
        return Ok(true);
    };

    match msg {
        AdminConStatMsg::Add {
            connection_key,
            tx,
            addr,
            ws_key,
        } => {
            // let statistics = vec![Statistic::new(addr.clone().to_string())];

            let current_con_stats: AdminStat = stats
                .entry(connection_key.clone())
                .or_insert(AdminStat::new(addr.clone(), false, HashMap::new()))
                .clone();

            let msg = ServerMsg::AdminStats(AdminStatsRes::Started(stats.clone()));
            let msg: WsPackage<ServerMsg> = (ws_key.clone(), msg);
            trace!("admin stats: sending: {:?}", &msg);
            let msg = ServerMsg::as_bytes(msg)?;
            let msg = Message::binary(msg);
            tx.send(ConMsg::Send(msg)).await?;
            list.insert(connection_key.clone(), (ws_key, tx));

            let update_msg = ServerMsg::AdminStats(AdminStatsRes::UpdateAddedNew {
                con_key: connection_key,
                stat: current_con_stats,
            });
            for (con_key, (ws_key, tx)) in list {
                let update_msg: WsPackage<ServerMsg> = (ws_key.clone(), update_msg.clone());
                let update_msg = ServerMsg::as_bytes(update_msg)?;
                let update_msg = Message::binary(update_msg);
                tx.send(ConMsg::Send(update_msg.clone())).await;
            }
        }
        AdminConStatMsg::Remove { key } => {
            list.remove(&key);
        }
    }

    Ok(false)
}
