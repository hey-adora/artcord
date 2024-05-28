use crate::ws::con::req::res::ResErr;
use crate::ws::{GlobalConChannel, WsAppMsg};
use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::{
    ClientMsg, ClientPathType, ClientThresholdMiddleware,
};
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::misc::throttle_connection::{IpBanReason, LiveThrottleConnectionCount};
use artcord_state::misc::throttle_threshold::Threshold;
use artcord_state::model::ws_statistics::{TempConIdType, WsStat};
use artcord_state::util::time::TimeMiddleware;
use chrono::{DateTime, TimeDelta, Utc};
use enum_index::EnumIndex;
use futures::stream::{SplitSink, SplitStream};
use futures::SinkExt;
use futures::StreamExt;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, trace, Instrument};

use self::req::req_task;
use self::throttle_stats_listener_tracker::{ConTrackerErr, ThrottleStatsListenerTracker};

pub mod req;
pub mod throttle_stats_listener_tracker;

#[derive(Debug, Clone)]
pub enum IpConMsg {
    // AddIpStatListener {
    //     msg_author: TempConIdType,
    //     con_id: TempConIdType,
    //     con_tx: mpsc::Sender<ConMsg>,
    //     current_state_tx: mpsc::Sender<HashMap<ClientPathType, WsReqStat>>,
    // },
}

#[derive(Debug, Clone)]
pub enum GlobalConMsg {
    AddIpStatListener {
        con_id: TempConIdType,
        con_tx: mpsc::Sender<ConMsg>,
        ws_key: WsRouteKey,
    },
    RemoveIpStatListener {
        con_id: TempConIdType,
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
    AddWsThrottleListener {
        //msg_author: TempConIdType,
        //con_id: TempConIdType,
        //con_tx: mpsc::Sender<ConMsg>,
        res_key: WsRouteKey,
        //current_state_tx: oneshot::Sender<Vec<WsStat>>,
    },
    RemoveWsThrottleListener,
    // AddWsThrottleListener {
    //     msg_author: TempConIdType,
    //     con_id: TempConIdType,
    //     con_tx: mpsc::Sender<ConMsg>,
    //     current_state_tx: mpsc::Sender<HashMap<ClientPathType, WsReqStat>>,
    // },
    // RemoveWsThrottleListener {
    //     msg_author: TempConIdType,
    // },
    // AddReqThrottleListener {
    //     msg_author: TempConIdType,
    //     con_tx: mpsc::Sender<ConMsg>,
    //     current_state_tx: mpsc::Sender<HashMap<ClientPathType, WsReqStat>>,
    // },
}

#[derive(Debug)]
pub struct Con<
    TimeMiddlewareType: TimeMiddleware + Clone + Sync + Send + 'static,
    ThresholdMiddlewareType: ClientThresholdMiddleware + Send + Clone + Sync + 'static,
> {
    con_id: TempConIdType,
    ws_stream_tx: SplitSink<WebSocketStream<TcpStream>, Message>,
    ws_stream_rx: SplitStream<WebSocketStream<TcpStream>>,
    ip_con_tx: broadcast::Sender<IpConMsg>,
    ip_con_rx: broadcast::Receiver<IpConMsg>,
    global_con_tx: broadcast::Sender<GlobalConMsg>,
    global_con_rx: broadcast::Receiver<GlobalConMsg>,
    con_tx: mpsc::Sender<ConMsg>,
    con_rx: mpsc::Receiver<ConMsg>,
    ws_app_tx: mpsc::Sender<WsAppMsg>,
    cancellation_token: CancellationToken,
    db: Arc<DB>,
    ip: IpAddr,
    addr: SocketAddr,
    stats: WsStat,
    listener_tracker: ThrottleStatsListenerTracker,
    is_listening: bool,
    ban_threshold: Threshold,
    ban_duration: TimeDelta,
    //banned_until: Option<(DateTime<Utc>, IpBanReason)>,
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
        ip_con_tx: broadcast::Sender<IpConMsg>,
        ip_con_rx: broadcast::Receiver<IpConMsg>,
        ban_threshold: Threshold,
        ban_duration: TimeDelta,
        time_middleware: TimeMiddlewareType,
        threshold_middleware: ThresholdMiddlewareType,
        listener_tracker: ThrottleStatsListenerTracker,
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
        let time = time_middleware.get_time().await;
        let (ws_stream_write, ws_stream_read) = ws_stream.split();
        let (con_tx, mut con_rx) = mpsc::channel::<ConMsg>(1);
        let con_id: TempConIdType = uuid::Uuid::new_v4().as_u128();

        let mut con = Self {
            stats: WsStat::new(con_id, ip.clone(), addr.clone(), time),
            con_id,
            ws_stream_tx: ws_stream_write,
            ws_stream_rx: ws_stream_read,
            ip_con_tx,
            ip_con_rx,
            global_con_tx,
            global_con_rx,
            con_tx,
            con_rx,
            ws_app_tx,
            cancellation_token,
            db,
            ip,
            addr,
            listener_tracker,
            is_listening: false,
            ban_threshold,
            ban_duration,
            //banned_until: None,
            con_task_tracker: TaskTracker::new(),
            time_middleware,
            threshold_middleware,
        };

        con.run().await;
    }

    pub async fn run(&mut self) {
        let result = self.prepare().await;
        if let Err(err) = result {
            error!("con on prepare err: {}", err);
        }

        loop {
            select! {

                msg = self.ws_stream_rx.next() => {
                    trace!("recv msg from stream");
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
                    trace!("recv msg from stream finished");
                },
                msg = self.ip_con_rx.recv() => {
                    trace!("recv msg from ip con: {:#?}", &msg);
                    let msg = match msg {
                        Ok(msg) => msg,
                        Err(err) => {
                            error!("error receiving from global con channel: {}", err);
                            continue;
                        }
                    };


                    let exit = self.on_ip_msg(msg).await;
                    let exit = match exit {
                        Ok(exit) => exit,
                        Err(err) => {
                            error!("req on msg err: {}", err);
                            continue;
                        }
                    };
                    trace!("recv msg from ip con finished");
                    if exit {
                        break;
                    }


                },
                msg = self.global_con_rx.recv() => {
                    trace!("recv msg from global con: {:#?}", &msg);
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
                    trace!("recv msg from global con finished");
                    if exit {
                        break;
                    }

                },
                msg = self.con_rx.recv() => {

                    trace!("recv msg from con_rx");
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
                    trace!("recv msg from finished");
                    if exit {
                        break;
                    }
                },


                _ = self.cancellation_token.cancelled() => {
                    break;
                }
            }
        }

        self.on_disconnect().await;
    }

    pub async fn on_msg(&mut self, msg: ConMsg) -> Result<bool, ConErr> {
        match msg {
            ConMsg::RemoveWsThrottleListener => {
                self.ws_app_tx
                    .send(WsAppMsg::RemoveListener {
                        con_id: self.con_id,
                    })
                    .await?;
                self.global_con_tx
                    .send(GlobalConMsg::RemoveIpStatListener {
                        con_id: self.con_id,
                    })?;
            }
            ConMsg::AddWsThrottleListener { res_key: ws_key } => {
                //let self.stats_listeners.cons.contains_key(&self.con_id);
                //let result = self.stats_listeners.cons.insert(self.con_id, (ws_key, self.con_tx.clone()));
                // if result
                // let mut stats: Vec<WsStat> = Vec::new();
                // let (current_global_state_tx, mut current_global_state_rx) = mpsc::channel::<WsStat>(100);
                let (done_tx, done_rx) = oneshot::channel();
                self.ws_app_tx
                    .send(WsAppMsg::AddListener {
                        con_id: self.con_id,
                        con_tx: self.con_tx.clone(),
                        ws_key,
                        done_tx,
                    })
                    .await?;
                let con_stats = done_rx.await.map_err(ConErr::DoneTxErr)?;
                self.global_con_tx.send(GlobalConMsg::AddIpStatListener {
                    ws_key,
                    con_id: self.con_id,
                    con_tx: self.con_tx.clone(),
                })?;

                let msg = ServerMsg::WsLiveStatsIpConnections(con_stats);
                let msg: WsPackage<ServerMsg> = (ws_key, msg);
                let msg = ServerMsg::as_bytes(msg)?;
                let msg = Message::Binary(msg);
                let send_result = self.ws_stream_tx.send(msg).await;
                if let Err(err) = send_result {
                    debug!("failed to send msg: {}", err);
                    return Ok(true);
                }

                // while let Some(ws_stat) = current_global_state_rx.recv().await {
                //     stats.push(ws_stat);
                // }

                //current_state_tx.send(vec![self.stats.clone()]).map_err(|_| ReqOnMsgErr::LiveStatsSend)?;
            }
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
                        &time,
                    )
                    .await;
                let result = match result {
                    artcord_state::misc::throttle_threshold::AllowCon::Allow => {
                        if !self.listener_tracker.cons.is_empty() {
                            if let Some(stat) = self.stats.count.get(&path) {
                                self.listener_tracker
                                    .send(ServerMsg::WsLiveStatsConReqAllowed {
                                        con_id: self.con_id,
                                        path,
                                        total_amount: stat.total_allowed_count,
                                    })
                                    .await?;
                            } else {
                                error!(
                                    "failed to send path update, missing ip entry for: {} in {:#?}",
                                    &path, &self.stats
                                );
                            }
                        }

                        true
                    }
                    artcord_state::misc::throttle_threshold::AllowCon::AlreadyBanned => false,
                    artcord_state::misc::throttle_threshold::AllowCon::Banned((date, reason)) => {
                        if !self.listener_tracker.cons.is_empty() {
                            if let Some(stat) = self.stats.count.get(&path) {
                                self.listener_tracker
                                    .send(ServerMsg::WsLiveStatsIpBanned {
                                        ip: self.ip,
                                        date,
                                        reason,
                                    })
                                    .await?;

                                self.listener_tracker
                                    .send(ServerMsg::WsLiveStatsConReqBanned {
                                        con_id: self.con_id,
                                        path,
                                        total_amount: stat.total_banned_count,
                                    })
                                    .await?;
                            } else {
                                error!(
                                    "failed to send path update, missing ip entry for: {} in {:#?}",
                                    &path, &self.stats
                                );
                            }
                        }

                        false
                    }
                    artcord_state::misc::throttle_threshold::AllowCon::Blocked => {
                        if !self.listener_tracker.cons.is_empty() {
                            if let Some(stat) = self.stats.count.get(&path) {
                                self.listener_tracker
                                    .send(ServerMsg::WsLiveStatsConReqBlocked {
                                        con_id: self.con_id,
                                        path,
                                        total_amount: stat.total_blocked_count,
                                    })
                                    .await?;
                            } else {
                                error!(
                                    "failed to send path update, missing ip entry for: {} in {:#?}",
                                    &path, &self.stats
                                );
                            }
                        }
                        false
                    }
                    artcord_state::misc::throttle_threshold::AllowCon::Unbanned => {
                        if !self.listener_tracker.cons.is_empty() {
                            if let Some(stat) = self.stats.count.get(&path) {
                                self.listener_tracker
                                    .send(ServerMsg::WsLiveStatsIpUnbanned { ip: self.ip })
                                    .await?;
                                self.listener_tracker
                                    .send(ServerMsg::WsLiveStatsConReqAllowed {
                                        con_id: self.con_id,
                                        path,
                                        total_amount: stat.total_allowed_count,
                                    })
                                    .await?;
                            } else {
                                error!(
                                    "failed to send path update, missing ip entry for: {} in {:#?}",
                                    &path, &self.stats
                                );
                            }
                        }

                        true
                    }
                };
                allow_tx
                    .send(result)
                    .map_err(|_| ConErr::ThrottleCheckSend)?;

                //let stats = &mut *ip_stats_rx.borrow_mut();
            } // ConMsg::AddReqThrottleListener {
              //     msg_author,
              //     con_tx,
              //     current_state_tx,
              // } => {
              //     if self.con_id == msg_author {
              //         return Ok(false);
              //     }
              //     current_state_tx.send(self.stats.paths.clone()).await?;
              //     self.stats_listeners.insert(msg_author, con_tx);
              // }
        }
        Ok(false)
    }

    pub async fn on_global_msg(&mut self, msg: GlobalConMsg) -> Result<bool, ConErr> {
        match msg {
            GlobalConMsg::AddIpStatListener {
                ws_key,
                con_id,
                con_tx,
            } => {
                //con_tx.send(ConMsg::Send(()))
                // if self.con_id == con_id {
                //     return Ok(false);
                // }
                self.is_listening = true;
                self.listener_tracker
                    .cons
                    .insert(con_id, (ws_key, con_tx.clone()));

                let msg = ServerMsg::WsLiveStatsConnected(self.stats.clone());
                let msg: WsPackage<ServerMsg> = (ws_key, msg);
                let msg = ServerMsg::as_bytes(msg)?;
                let msg = Message::Binary(msg);

                con_tx.send(ConMsg::Send(msg)).await?;
                // current_global_state_tx.
            }
            GlobalConMsg::RemoveIpStatListener { con_id } => {
                self.is_listening = false;
                self.listener_tracker.cons.remove(&con_id);
            }
        }
        Ok(false)
    }

    pub async fn on_ip_msg(&mut self, msg: IpConMsg) -> Result<bool, ConErr> {
        Ok(false)
    }

    pub async fn on_req(&mut self, msg: Message) {
        //debug!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let msg_name = match &msg {
            Message::Binary(_) => "binary",
            Message::Close(_) => "close",
            Message::Frame(_) => "frame",
            Message::Ping(_) => "ping",
            Message::Pong(_) => "pong",
            Message::Text(_) => "text",
        };

        self.con_task_tracker.spawn(
            req_task(
                msg,
                self.db.clone(),
                self.con_tx.clone(),
                self.ws_app_tx.clone(),
                self.con_id,
                self.addr,
                self.ip,
                self.threshold_middleware.clone(),
            )
            .instrument(tracing::trace_span!("req", "{}", msg_name,)),
        );
    }

    pub async fn prepare(&mut self) -> Result<(), ConErr> {
        if !self.listener_tracker.cons.is_empty() {
            self.listener_tracker
                .send(ServerMsg::WsLiveStatsConnected(self.stats.clone()))
                .await?;
        }
        Ok(())
    }

    pub async fn on_disconnect(&mut self) {
        trace!(
            "ws: user({} - {}): exiting..., tasks left: {}",
            self.ip,
            self.con_id,
            self.con_task_tracker.len()
        );
        self.con_rx.close();
        debug!("1");
        if !self.cancellation_token.is_cancelled() {
            debug!("2");
            if self.is_listening {
                debug!("3");
                let send_result = self
                    .ws_app_tx
                    .send(WsAppMsg::RemoveListener {
                        con_id: self.con_id,
                    })
                    .await;
                if let Err(err) = send_result {
                    error!("on disconnect error: {}", err);
                }
                debug!("4");
                let send_result = self.global_con_tx.send(GlobalConMsg::RemoveIpStatListener {
                    con_id: self.con_id,
                });
                debug!("6");
                if let Err(err) = send_result {
                    error!("on disconnect error: {}", err);
                }
            }
            debug!("7");
            if !self.listener_tracker.cons.is_empty() {
                debug!("8");
                if self.is_listening {
                    debug!("9");
                    self.listener_tracker.cons.remove(&self.con_id);
                }
                debug!("10");
                let send_result = self
                    .listener_tracker
                    .send(ServerMsg::WsLiveStatsDisconnected {
                        con_id: self.con_id,
                    })
                    .await;
                if let Err(err) = send_result {
                    error!("on disconnect error: {}", err);
                }
            }
            debug!("11");
        }
        debug!("12");
        self.con_task_tracker.close();
        debug!("13");
        self.con_task_tracker.wait().await;
        debug!("14");
        //trace!("disconnected");
        let send_result = self
            .ws_app_tx
            .send(WsAppMsg::Disconnected { ip: self.ip })
            .await;

        if let Err(err) = send_result {
            error!("on disconnect error: {}", err);
        }

        trace!("disconnected");
    }
}

#[derive(Error, Debug)]
pub enum ConErr {
    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("error from con tracker: {0}")]
    ConTrackerErr(#[from] ConTrackerErr),

    #[error("failed to send throttle check result.")]
    ThrottleCheckSend,

    #[error("failed to recv oneshot done_tx from ws.")]
    DoneTxErr(#[from] oneshot::error::RecvError),

    #[error("failed to send con msg: {0}")]
    SendConMsgErr(#[from] mpsc::error::SendError<ConMsg>),

    #[error("failed to send stats: {0}")]
    SendStatsErr(#[from] mpsc::error::SendError<WsStat>),

    #[error("failed to send ws_msg: {0}")]
    SendWsMsgErr(#[from] mpsc::error::SendError<WsAppMsg>),

    #[error("failed to send global con msg: {0}")]
    SendGlobalConErr(#[from] broadcast::error::SendError<GlobalConMsg>),
}
