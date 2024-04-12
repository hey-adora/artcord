use std::{collections::HashMap, net::SocketAddr};

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_state::{
    message::{
        prod_perm_key::ProdMsgPermKey,
        prod_server_msg::{AdminStatsRes, ServerMsg},
    },
    model::statistics::Statistic,
};
use tokio::{select, sync::mpsc, task::JoinHandle};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, trace};

use super::{ws_throttle::ThrottleListenerMsgError, ConMsg};

pub enum WsThrottleListenerMsg {
    Add {
        connection_key: uuid::Uuid,
        tx: mpsc::Sender<ConMsg>,
        addr: SocketAddr,
        ws_key: WsRouteKey<u128, ProdMsgPermKey>,
    },

    Remove {
        key: uuid::Uuid,
    },
}

pub async fn create_throttle_listener_task(
    task_tracker: &TaskTracker,
    cancelation_token: &CancellationToken,
) -> (JoinHandle<()>, mpsc::Sender<WsThrottleListenerMsg>) {
    let (tx, rx) = mpsc::channel::<WsThrottleListenerMsg>(100);
    let listener_task = task_tracker.spawn(throttle_listener_task(rx, cancelation_token.clone()));
    (listener_task, tx)
}

pub async fn throttle_listener_task(
    mut rx: mpsc::Receiver<WsThrottleListenerMsg>,
    cancelation_token: CancellationToken,
) {
    let mut throttle_listener_list: HashMap<uuid::Uuid, (SocketAddr, mpsc::Sender<ConMsg>)> =
        HashMap::new();
    loop {
        select! {
            msg = rx.recv() => {
                let exit = on_throttle_listener_msg(msg, &mut throttle_listener_list).await;
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

pub async fn on_throttle_listener_msg(
    msg: Option<WsThrottleListenerMsg>,
    list: &mut HashMap<uuid::Uuid, (SocketAddr, mpsc::Sender<ConMsg>)>,
) -> Result<bool, ThrottleListenerMsgError> {
    let Some(msg) = msg else {
        return Ok(true);
    };

    match msg {
        WsThrottleListenerMsg::Add {
            connection_key,
            tx,
            addr,
            ws_key,
        } => {
            pub enum WsThrottleListenerMsg {
                Add {
                    connection_key: uuid::Uuid,
                    tx: mpsc::Sender<ConMsg>,
                    addr: SocketAddr,
                    ws_key: WsRouteKey<u128, ProdMsgPermKey>,
                },

                Remove {
                    key: uuid::Uuid,
                },
            }

            pub async fn create_throttle_listener_task(
                task_tracker: &TaskTracker,
                cancelation_token: &CancellationToken,
            ) -> (JoinHandle<()>, mpsc::Sender<WsThrottleListenerMsg>) {
                let (tx, rx) = mpsc::channel::<WsThrottleListenerMsg>(100);
                let listener_task =
                    task_tracker.spawn(throttle_listener_task(rx, cancelation_token.clone()));
                (listener_task, tx)
            }

            pub async fn throttle_listener_task(
                mut rx: mpsc::Receiver<WsThrottleListenerMsg>,
                cancelation_token: CancellationToken,
            ) {
                let mut throttle_listener_list: HashMap<
                    uuid::Uuid,
                    (SocketAddr, mpsc::Sender<ConMsg>),
                > = HashMap::new();
                loop {
                    select! {
                        msg = rx.recv() => {
                            let exit = on_throttle_listener_msg(msg, &mut throttle_listener_list).await;
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

            pub async fn on_throttle_listener_msg(
                msg: Option<WsThrottleListenerMsg>,
                list: &mut HashMap<uuid::Uuid, (SocketAddr, mpsc::Sender<ConMsg>)>,
            ) -> Result<bool, ThrottleListenerMsgError> {
                let Some(msg) = msg else {
                    return Ok(true);
                };

                match msg {
                    WsThrottleListenerMsg::Add {
                        connection_key,
                        tx,
                        addr,
                        ws_key,
                    } => {
                        let statistics = vec![Statistic::new(addr.clone().to_string())];
                        let msg = ServerMsg::AdminStats(AdminStatsRes::Started(statistics));
                        let msg = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
                            key: ws_key,
                            data: msg,
                        };
                        let msg = ServerMsg::as_bytes(msg)?;
                        let msg = Message::binary(msg);
                        tx.send(ConMsg::Send(msg)).await?;
                        list.insert(connection_key, (addr, tx));
                    }
                    WsThrottleListenerMsg::Remove { key } => {
                        list.remove(&key);
                    }
                }

                Ok(false)
            }

            let statistics = vec![Statistic::new(addr.clone().to_string())];
            let msg = ServerMsg::AdminStats(AdminStatsRes::Started(statistics));
            let msg = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
                key: ws_key,
                data: msg,
            };
            let msg = ServerMsg::as_bytes(msg)?;
            let msg = Message::binary(msg);
            tx.send(ConMsg::Send(msg)).await?;
            list.insert(connection_key, (addr, tx));
        }
        WsThrottleListenerMsg::Remove { key } => {
            list.remove(&key);
        }
    }

    Ok(false)
}
