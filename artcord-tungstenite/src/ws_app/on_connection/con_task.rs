use crate::ws_app::on_connection::con_task::on_msg::on_msg;
use crate::ws_app::on_connection::con_task::on_req::on_req;
use crate::ws_app::ws_statistic::WsStatsMsg;
use crate::ws_app::{GlobalConChannel, PathStats, WsAppMsg};
use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::{ClientPathType, ClientThresholdMiddleware};
use artcord_state::misc::throttle_connection::{
    IpBanReason,  LiveThrottleConnectionCount, PathStat,
};
use artcord_state::misc::throttle_threshold::Threshold;
use artcord_state::model::ws_statistics::TempConIdType;
use artcord_state::util::time::TimeMiddleware;
use chrono::{DateTime, TimeDelta, Utc};
use futures::stream::{SplitSink, SplitStream};
use futures::SinkExt;
use futures::StreamExt;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, trace};

use self::on_msg::ReqOnMsgErr;
use self::on_req::req_task::req_task;

pub mod on_msg;
pub mod on_req;

#[derive(Debug, Clone)]
pub enum GlobalConMsg {
    AddIpStatListener {
        msg_author: TempConIdType,
        con_tx: mpsc::Sender<ConMsg>,
        current_state_tx: mpsc::Sender<HashMap<ClientPathType, PathStat>>,
    },
}

#[derive(Debug)]
pub enum ConMsg {
    Send(Message),
    Stop,
    CheckThrottle {
        path: usize,
        block_threshold: Threshold,
        allow_tx: oneshot::Sender<bool>,
    },
    AddIpStatListener {
        msg_author: TempConIdType,
        con_tx: mpsc::Sender<ConMsg>,
        current_state_tx: mpsc::Sender<HashMap<ClientPathType, PathStat>>,
    },
}

#[derive(Debug)]
pub struct Con<
    TimeMiddlewareType: TimeMiddleware + Clone + Sync + Send + 'static,
    ThresholdMiddlewareType: ClientThresholdMiddleware + Send + Clone + Sync + 'static,
> {
    con_id: TempConIdType,
    ws_stream_tx: SplitSink<WebSocketStream<TcpStream>, Message>,
    ws_stream_rx: SplitStream<WebSocketStream<TcpStream>>,
    global_con_tx: broadcast::Sender<GlobalConMsg>,
    global_con_rx: broadcast::Receiver<GlobalConMsg>,
    con_tx: mpsc::Sender<ConMsg>,
    con_rx: mpsc::Receiver<ConMsg>,
    ws_app_tx: mpsc::Sender<WsAppMsg>,
    cancellation_token: CancellationToken,
    db: Arc<DB>,
    ip: IpAddr,
    addr: SocketAddr,
    stats: PathStats,
    stats_listeners: HashMap<TempConIdType, mpsc::Sender<ConMsg>>,
    ban_threshold: Threshold,
    ban_duration: TimeDelta,
    banned_until: Option<(DateTime<Utc>, IpBanReason)>,
    con_task_tracker: TaskTracker,
    time_middleware: TimeMiddlewareType,
    threshold_middleware: ThresholdMiddlewareType,
}

impl<
        TimeMiddlewareType: TimeMiddleware + Clone + Sync + Send + 'static,
        ThresholdMiddlewareType: ClientThresholdMiddleware + Send + Clone + Sync + 'static,
    > Con<TimeMiddlewareType, ThresholdMiddlewareType>
{
    pub async fn connect(
        stream: TcpStream,
        cancellation_token: CancellationToken,
        db: Arc<DB>,
        ws_app_tx: mpsc::Sender<WsAppMsg>,
        ip: IpAddr,
        addr: SocketAddr,
        //admin_ws_stats_tx: mpsc::Sender<WsStatsMsg>,
        (global_con_tx, mut global_con_rx): GlobalConChannel,
        ban_threshold: Threshold,
        ban_duration: TimeDelta,
        time_middleware: TimeMiddlewareType,
        threshold_middleware: ThresholdMiddlewareType,
    ) {
        trace!("task spawned!");
        let ws_stream = tokio_tungstenite::accept_async(stream).await;
        let ws_stream = match ws_stream {
            Ok(ws_stream) => ws_stream,
            Err(err) => {
                trace!("ws_error: {}", err);
                return;
            }
        };
        trace!("con accepted");
        let (ws_stream_write, ws_stream_read) = ws_stream.split();
        let (con_tx, mut con_rx) = mpsc::channel::<ConMsg>(1);
        let con_id: TempConIdType = uuid::Uuid::new_v4().as_u128();

        let mut con = Self {
            con_id,
            ws_stream_tx: ws_stream_write,
            ws_stream_rx: ws_stream_read,
            global_con_tx,
            global_con_rx,
            con_tx,
            con_rx,
            ws_app_tx,
            cancellation_token,
            db,
            ip,
            addr,
            stats: PathStats::new(),
            stats_listeners: HashMap::new(),
            ban_threshold,
            ban_duration,
            banned_until: None,
            con_task_tracker: TaskTracker::new(),
            time_middleware,
            threshold_middleware,
        };

        con.run().await;
    }

    pub async fn run(&mut self) {
        loop {
            select! {

                msg = self.ws_stream_rx.next() => {
                    let Some(msg) = msg else {
                        trace!("connection msg channel closed");
                        break;
                    };

                    let msg = match msg {
                        Ok(msg) => msg,
                        Err(err) => {
                            error!("error receiving from stream: {}", err);
                            continue;
                        }
                    };

                    self.on_req(msg).await;
                },
                msg = self.global_con_rx.recv() => {
                    let msg = match msg {
                        Ok(msg) => msg,
                        Err(err) => {
                            error!("error receiving from global con channel: {}", err);
                            continue;
                        }
                    };


                    let exit = self.on_global_msg(msg).await;
                    let exit = match exit {
                        Ok(exit) => exit,
                        Err(err) => {
                            error!("req on msg err: {}", err);
                            continue;
                        }
                    };
                    if exit {
                        break;
                    }
                },
                msg = self.con_rx.recv() => {
                    let Some(msg) = msg else {
                        trace!("connection msg channel closed");
                        break;
                    };

                    let exit = self.on_msg(msg).await;
                    let exit = match exit {
                        Ok(exit) => exit,
                        Err(err) => {
                            error!("req on msg err: {}", err);
                            continue;
                        }
                    };
                    if exit {
                        break;
                    }
                },

                _ = self.cancellation_token.cancelled() => {
                    break;
                }
            }
        }

        self.on_disconnect();
    }

    pub async fn on_msg(&mut self, msg: ConMsg) -> Result<bool, ReqOnMsgErr> {
        match msg {
            ConMsg::Send(msg) => {
                let send_result = self.ws_stream_tx.send(msg).await;
                if let Err(err) = send_result {
                    debug!("failed to send msg: {}", err);
                    return Ok(true);
                }
            }
            ConMsg::Stop => {
                return Ok(true);
            }
            ConMsg::CheckThrottle {
                path,
                block_threshold,
                allow_tx,
            } => {
                let time = self.time_middleware.get_time().await;
                let result = self
                    .stats
                    .inc_path(
                        path,
                        block_threshold,
                        &self.ban_threshold,
                        &self.ban_duration,
                        &mut self.banned_until,
                        &time,
                    )
                    .await;
                let result = match result {
                    artcord_state::misc::throttle_threshold::AllowCon::Allow => true,
                    artcord_state::misc::throttle_threshold::AllowCon::AlreadyBanned => false,
                    artcord_state::misc::throttle_threshold::AllowCon::Banned(_) => false,
                    artcord_state::misc::throttle_threshold::AllowCon::Blocked => false,
                    artcord_state::misc::throttle_threshold::AllowCon::Unbanned => true,
                };
                allow_tx
                    .send(result)
                    .map_err(|_| ReqOnMsgErr::ThrottleCheckSend)?;
                //let stats = &mut *ip_stats_rx.borrow_mut();
            }
            ConMsg::AddIpStatListener {
                msg_author,
                con_tx,
                current_state_tx,
            } => {
                if self.con_id == msg_author {
                    return Ok(false);
                }
                current_state_tx.send(self.stats.paths.clone()).await?;
                self.stats_listeners.insert(msg_author, con_tx);
            }
        }
        Ok(false)
    }

    pub async fn on_global_msg(&mut self, msg: GlobalConMsg) -> Result<bool, ReqOnMsgErr> {
        Ok(false)
    }

    pub async fn on_req(&mut self, msg: Message) {
        self.con_task_tracker.spawn(req_task(
            msg,
            self.db.clone(),
            self.con_tx.clone(),
            self.ws_app_tx.clone(),
            self.con_id,
            self.addr,
            self.ip,
            self.threshold_middleware.clone(),
        ));
    }

    pub async fn on_disconnect(&mut self) {
        debug!(
            "ws: user({}): exiting..., tasks left: {}",
            self.ip,
            self.con_task_tracker.len()
        );

        self.con_task_tracker.close();
        self.con_task_tracker.wait().await;
        trace!("disconnected");
        let send_result = self
            .ws_app_tx
            .send(WsAppMsg::Disconnected {
                ip: self.ip,
                connection_key: self.con_id,
            })
            .await;
        if let Err(err) = send_result {
            error!("failed to send disconnect to ws: {}", err);
        }
        trace!("disconnected");
    }
}

// pub async fn con_task(
//     stream: TcpStream,
//     cancellation_token: CancellationToken,
//     db: Arc<DB>,
//     ws_app_tx: mpsc::Sender<WsAppMsg>,
//     ip: IpAddr,
//     addr: SocketAddr,
//     admin_ws_stats_tx: mpsc::Sender<WsStatsMsg>,
//     get_threshold: impl ClientThresholdMiddleware + Send + Sync + Clone + 'static,
//     (global_con_channel_tx, mut global_con_channel_rx): GlobalConChannel,
// ) {
//     trace!("task spawned!");
//     let ws_stream = tokio_tungstenite::accept_async(stream).await;
//     // .expect("Error during the websocket handshake occurred.");
//     let ws_stream = match ws_stream {
//         Ok(ws_stream) => ws_stream,
//         Err(err) => {
//             trace!("ws_error: {}", err);
//             return;
//         }
//     };
//     // ws_stream.
//     trace!("con accepted");
//     let con_id: TempConIdType = uuid::Uuid::new_v4().as_u128();
//     // let Ok(ws_stream) = ws_stream else {
//     //     return;
//     // };

//     let (connection_task_tx, mut connection_task_rx) = mpsc::channel::<ConMsg>(1);
//     // let (client_out_handler, mut client_out_handle) = mpsc::channel::<Message>(10);
//     let (mut client_out, mut client_in) = ws_stream.split();
//     let user_task_tracker: TaskTracker = TaskTracker::new();

//     let mut stats = PathStats::new();
//     let mut ip_stats_listeners: HashMap<TempConIdType, mpsc::Sender<ConMsg>> = HashMap::new();

//     // let send_result = admin_ws_stats_tx
//     //     .send(WsStatsMsg::AddTrack {
//     //         connection_key: con_id.clone(),
//     //         tx: connection_task_tx.clone(),
//     //         ip,
//     //         addr,
//     //     })
//     //     .await;
//     // if let Err(err) = send_result {
//     //     error!("error adding track: {}", err);
//     // }

//     // let read = async {};

//     // let write = async {
//     //     while let Some(msg) = .await {
//     //         let send_result = client_out.send(msg).await;
//     //         if let Err(send_result) = send_result {
//     //             error!("send error: {}", send_result);
//     //             return;
//     //         }
//     //     }
//     // };

//     // pin_mut!(read, write);

//     loop {
//         select! {

//             result = client_in.next() => {
//                 trace!("read finished");
//                 let exit = on_req(result, &user_task_tracker, &db, &connection_task_tx, &admin_ws_stats_tx, &ws_app_tx, &con_id, &addr, &ip, &get_threshold).await;
//                 if exit {
//                     break;
//                 }
//             },

//             // result = client_out_handle.recv() => {
//             //     trace!("write finished");
//             //     let exit = request_write_task(&mut client_out, result).await;
//             //     if exit {
//             //         break;
//             //     }
//             // },
//             result = global_con_channel_rx.recv() => {
//                 let result = match result {
//                     Ok(result) => result,
//                     Err(err) => {
//                         error!("error receiving from global con channel: {}", err);
//                         continue;
//                     }
//                 };
//                 let exit = on_msg(Some(result), &con_id, &mut client_out, &global_con_channel_tx, &mut ip_stats_listeners, &mut stats).await;
//                 let exit = match exit {
//                     Ok(exit) => exit,
//                     Err(err) => {
//                         error!("req on msg err: {}", err);
//                         continue;
//                     }
//                 };
//                 if exit {
//                     break;
//                 }
//             },
//             result = connection_task_rx.recv() => {
//                 let exit = on_msg(result, &con_id, &mut client_out, &global_con_channel_tx, &mut ip_stats_listeners, &mut stats).await;
//                 let exit = match exit {
//                     Ok(exit) => exit,
//                     Err(err) => {
//                         error!("req on msg err: {}", err);
//                         continue;
//                     }
//                 };
//                 if exit {
//                     break;
//                 }
//             },

//             _ = cancellation_token.cancelled() => {
//                 break;
//             }
//         }
//     }

//     // let (finished_tx, finished_rx) = oneshot::channel();
//     // let send_result = admin_ws_stats_tx
//     //     .send(WsStatsMsg::StopTrack {
//     //         connection_key: con_id.clone(),
//     //         finished_tx
//     //     })
//     //     .await;
//     // if let Err(err) = send_result {
//     //     error!("error stoping track: {}", err);
//     // }
//     // let send_result = finished_rx.await;
//     // if let Err(err) = send_result {
//     //     error!("error stoping track: {}", err);
//     // }

//     // let send_result = ws_app_tx
//     //     .send(WsAppMsg::Disconnected { connection_key: con_id, ip })
//     //     .await;
//     // if let Err(err) = send_result {
//     //     error!("error sending disc to ws_app: {}", err);
//     // }

//     debug!(
//         "ws: user({}): exiting..., tasks left: {}",
//         ip,
//         user_task_tracker.len()
//     );
//     user_task_tracker.close();
//     user_task_tracker.wait().await;
//     trace!("disconnected");
//     let send_result = ws_app_tx
//         .send(WsAppMsg::Disconnected {
//             ip,
//             connection_key: con_id,
//         })
//         .await;
//     if let Err(err) = send_result {
//         error!("failed to send disconnect to ws: {}", err);
//     }
//     trace!("disconnected");
// }
