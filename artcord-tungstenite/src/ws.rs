use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use artcord_leptos_web_sockets::WsRouteKey;
use artcord_mongodb::database::DBError;
use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::ClientPathType;
use artcord_state::message::prod_client_msg::ClientThresholdMiddleware;
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::misc::throttle_connection::IpBanReason;
use artcord_state::misc::throttle_connection::LiveThrottleConnectionCount;
use artcord_state::misc::throttle_connection::TempThrottleConnection;
use artcord_state::misc::throttle_threshold::AllowCon;
use artcord_state::misc::throttle_threshold::Threshold;
use artcord_state::misc::throttle_threshold::ThrottleRanged;
use artcord_state::misc::throttle_threshold::ThrottleSimple;
use artcord_state::model::ws_statistics;
use artcord_state::model::ws_statistics::DbReqStat;
use artcord_state::model::ws_statistics::TempConIdType;
use artcord_state::util::time::time_is_past;
use artcord_state::util::time::time_passed_days;
use artcord_state::util::time::TimeMiddleware;
use artcord_state::ws::WsIpStat;
use chrono::DateTime;
use chrono::Month;
use chrono::Months;
use chrono::TimeDelta;
use chrono::Utc;
use futures::join;
use futures::pin_mut;
use futures::stream::SplitSink;
use futures::stream::SplitStream;
use futures::FutureExt;
use futures::TryStreamExt;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::task;

use cfg_if::cfg_if;
use futures::future;
use futures::SinkExt;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time;
use tokio::time::sleep;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::http::response;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::debug;
use tracing::info;
use tracing::instrument;
use tracing::Instrument;
use tracing::{error, trace};

use crate::ws::con::GlobalConMsg;
use crate::ws::throttle::WsThrottle;
use crate::WsThreshold;

use self::con::throttle_stats_listener_tracker::ConTrackerErr;
use self::con::throttle_stats_listener_tracker::ThrottleStatsListenerTracker;
use self::con::Con;
use self::con::ConMsg;
use self::con::IpConMsg;
use self::con::IpManagerMsg;

pub mod con;
pub mod throttle;

pub type GlobalConChannel = (
    broadcast::Sender<GlobalConMsg>,
    broadcast::Receiver<GlobalConMsg>,
);

#[derive(Debug)]
pub enum WsAppMsg {
    Stop,
    Ban {
        ip: IpAddr,
        date: DateTime<Utc>,
        reason: IpBanReason,
    },
    // UnBan {
    //     ip: IpAddr,
    // },
    Disconnected {
      //  connection_key: TempConIdType,
        ip: IpAddr,
    },
    AddListener {
        con_id: TempConIdType,
        con_tx: mpsc::Sender<ConMsg>,
        ws_key: WsRouteKey,
        done_tx: oneshot::Sender<Vec<WsIpStat>>,
    },
    RemoveListener {
        con_id: TempConIdType,
        // tx: mpsc::Sender<ConMsg>,
        // ws_key: WsRouteKey,
    },
    // Inc {
    //     ip: IpAddr,
    //     path: ClientPathType,
    // },
}

pub struct Ws<
    TimeMiddlewareType: TimeMiddleware + Clone + Sync + Send + 'static,
    ThresholdMiddlewareType: ClientThresholdMiddleware + Send + Clone + Sync + 'static,
    SocketAddrMiddlewareType: GetUserAddrMiddleware + Send + Sync + Clone + 'static,
> {
    tcp_listener: TcpListener,
    ws_task_tracker: TaskTracker,
    root_task_tracker: TaskTracker,
    root_cancellation_token: CancellationToken,
    ws_addr: String,
    ws_threshold: WsThreshold,
    // ws_tx: mpsc::Sender<WsAppMsg>,
    // ws_rx: mpsc::Receiver<WsAppMsg>,
    global_con_tx: broadcast::Sender<GlobalConMsg>,
    global_con_rx: broadcast::Receiver<GlobalConMsg>,
    db: Arc<DB>,
    throttle: WsThrottle,
    listener_tracker: ThrottleStatsListenerTracker,
    time_middleware: TimeMiddlewareType,
    threshold_middleware: ThresholdMiddlewareType,
    socket_middleware: SocketAddrMiddlewareType,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ProdUserAddrMiddleware;

pub trait GetUserAddrMiddleware {
    fn get_addr(&self, addr: SocketAddr) -> impl std::future::Future<Output = SocketAddr> + Send;
}

impl<
        TimeMiddlewareType: TimeMiddleware + Clone + Sync + Send + 'static,
        ThresholdMiddlewareType: ClientThresholdMiddleware + Send + Clone + Sync + 'static,
        SocketAddrMiddlewareType: GetUserAddrMiddleware + Send + Sync + Clone + 'static,
    > Ws<TimeMiddlewareType, ThresholdMiddlewareType, SocketAddrMiddlewareType>
{
    pub async fn create(
        root_task_tracker: TaskTracker,
        root_cancellation_token: CancellationToken,
        ws_addr: String,
        ws_threshold: WsThreshold,
        db: Arc<DB>,
        time_middleware: TimeMiddlewareType,
        threshold_middleware: ThresholdMiddlewareType,
        socket_middleware: SocketAddrMiddlewareType,
    ) {
        let try_socket = TcpListener::bind(ws_addr.clone()).await;
        let tcp_listener = try_socket.expect("Failed to bind");

        let (global_con_tx, global_con_rx) = broadcast::channel::<GlobalConMsg>(1000);

        let mut ws = Self {
            tcp_listener,
            ws_task_tracker: TaskTracker::new(),
            root_task_tracker,
            root_cancellation_token,
            ws_addr,
            ws_threshold,
            // ws_tx,
            // ws_rx,
            global_con_tx,
            global_con_rx,
            db,
            throttle: WsThrottle::new(),
            listener_tracker: ThrottleStatsListenerTracker::new(),
            time_middleware,
            threshold_middleware,
            socket_middleware,
        };

        ws.run().await;
    }

    pub async fn run(&mut self) {
        info!("ws started");
        let (ws_tx, mut ws_rx) = mpsc::channel::<WsAppMsg>(1);

        //self.root_cancellation_token.is_cancelled()
        loop {
            select! {
                
                _ = self.root_cancellation_token.cancelled() => {
                    debug!("ws canceled");
                    break;
                }

                con = self.tcp_listener.accept() => {
                    trace!("con accepted");
                    let result = self.on_con(&ws_tx, con).await;
                    if let Err(err) = result {
                        error!("on_con err: {}", err);
                    }
                },

                ws_msg = ws_rx.recv() => {
                    trace!("ws recved msg: {:#?}", &ws_msg);
                    let exit = self.on_msg(ws_msg).await;
                    trace!("ws recved msg finished");
                    let exit = match exit {
                        Ok(exit) => exit,
                        Err(err) => {
                            error!("ws_app: on_ws_msg error: {}", err);
                            continue;
                        }
                    };
                    if exit {
                        debug!("ws_rx closed");
                        break;
                    }
                },

             
            }
        }
        trace!("ws app exiting...");
        self.ws_task_tracker.close();
        drop(ws_tx);
        loop {
            select! {
                // _ =  self.ws_task_tracker.wait() => {
                //     trace!("ws app exiting... all tasks closed");
                //     break;
                // }
                ws_msg = ws_rx.recv() => {
                    let exit = self.on_msg(ws_msg).await;
                    let exit = match exit {
                        Ok(exit) => exit,
                        Err(err) => {
                            error!("ws_app: on_ws_msg error: {}", err);
                            continue;
                        }
                    };
                    if exit {
                        trace!("ws app exiting... channel closed");
                        break;
                    }
                }
            }
        }

        debug!("ws app exited.");
    }

    pub async fn on_con(
        &mut self,
        ws_tx: &mpsc::Sender<WsAppMsg>,
        con: Result<(TcpStream, SocketAddr), io::Error>,
    ) -> Result<(), WsOnConErr> {
        let (stream, user_addr) = match con {
            Ok(result) => result,
            Err(err) => {
                debug!("ws({}): error accepting connection: {}", &self.ws_addr, err);
                return Ok(());
            }
        };

        let user_addr = self.socket_middleware.get_addr(user_addr).await;
        let ip = user_addr.ip();

        let time = self.time_middleware.get_time().await;

        let allow: bool = match self.throttle.inc_con(ip, &self.ws_threshold, &self.ws_task_tracker, &self.root_cancellation_token, &time, &self.time_middleware) {
            AllowCon::Allow => {
                if !self.listener_tracker.cons.is_empty() {
                    if let Some(total_amount) = self.throttle.get_total_allowed(&ip) {
                        let msg = ServerMsg::WsLiveStatsConAllowed { ip, total_amount };
                        self.listener_tracker.send(msg).await?;
                    } else {
                        error!("ws({}): missing ip entry: {}", &self.ws_addr, ip);
                    }
                }
                true
            }
            AllowCon::AlreadyBanned => false,
            AllowCon::Blocked => {
                if !self.listener_tracker.cons.is_empty() {
                    if let Some(total_amount) = self.throttle.get_total_blocked(&ip) {
                        let msg = ServerMsg::WsLiveStatsConBlocked { ip, total_amount };
                        self.listener_tracker.send(msg).await?;
                    } else {
                        error!("ws({}): missing ip entry: {}", &self.ws_addr, ip);
                    }
                }
                false
            }
            AllowCon::UnbannedAndBlocked => {
                if !self.listener_tracker.cons.is_empty() {
                    if let Some(total_amount) = self.throttle.get_total_blocked(&ip) {
                        let msg = ServerMsg::WsLiveStatsConBlocked { ip, total_amount };
                        self.listener_tracker.send(msg).await?;
                    } else {
                        error!("ws({}): missing ip entry: {}", &self.ws_addr, ip);
                    }

                    let msg = ServerMsg::WsLiveStatsIpUnbanned { ip };
                    self.listener_tracker.send(msg).await?;
                }
                self.throttle.unban_on_ip_manager(&ip).await?;
                false
            }
            AllowCon::UnbannedAndAllow => {
                if !self.listener_tracker.cons.is_empty() {
                    if let Some(total_amount) = self.throttle.get_total_allowed(&ip) {
                        let msg = ServerMsg::WsLiveStatsConAllowed { ip, total_amount };
                        self.listener_tracker.send(msg).await?;
                    } else {
                        error!("ws({}): missing ip entry: {}", &self.ws_addr, ip);
                    }
                    // if let Some(total_amount) = self.throttle.get_total_unbanned(&ip) {
                       
                    // } else {
                    //     error!("ws({}): missing ip entry: {}", &self.ws_addr, ip);
                    // }
                    // let msg = ServerMsg::WsLiveStatsIpUnbanned { ip };
                    // self.listener_tracker.send(msg).await?;

                    let msg = ServerMsg::WsLiveStatsIpUnbanned { ip };
                    self.listener_tracker.send(msg).await?;
                }
                self.throttle.unban_on_ip_manager(&ip).await?;
                true
            }
            AllowCon::Banned((date, reason)) => {
                if !self.listener_tracker.cons.is_empty() {
                    if let Some(total_amount) = self.throttle.get_total_banned(&ip) {
                        let msg = ServerMsg::WsLiveStatsConBanned { ip, total_amount };
                        self.listener_tracker.send(msg).await?;
                    } else {
                        error!("ws({}): missing ip entry: {}", &self.ws_addr, ip);
                    }

                    let msg = ServerMsg::WsLiveStatsIpBanned { ip, date, reason };
                    self.listener_tracker.send(msg).await?;
                }
                false
            }
        };
        
        if !allow {
            debug!("ws({}): dont connect", &self.ws_addr);
            return Ok(());
        };

        let Some((ip_con_tx, ip_con_rx, ip_data_sync_tx)) = self.throttle.get_ip_channel(&ip) else {
            error!("ws({}): failed to copy ip_con cahnnel", &self.ws_addr);
            return Ok(());
        };

        //let task_handle: JoinHandle<()> = self.ws_task_tracker.spawn(async {});

        self.ws_task_tracker.spawn(
            Con::connect(
                stream,
                self.root_cancellation_token.clone(),
                self.db.clone(),
                ws_tx.clone(),
                ip,
                user_addr,
                (self.global_con_tx.clone(), self.global_con_tx.subscribe()),
                ip_con_tx,
                ip_con_rx,
                ip_data_sync_tx,
                self.ws_threshold.ws_req_ban_threshold.clone(),
                self.ws_threshold.ws_req_ban_duration.clone(),
                self.time_middleware.clone(),
                self.threshold_middleware.clone(),
                self.listener_tracker.clone(),
            )
            .instrument(tracing::trace_span!(
                "con",
                "{}",
                user_addr.to_string()
            )),
        );

        Ok(())
    }

    pub async fn on_msg(&mut self, msg: Option<WsAppMsg>) -> Result<bool, WsMsgErr> {
        let time = self.time_middleware.get_time().await;

        let Some(msg) = msg else {
            return Ok(true);
        };

        match msg {
            WsAppMsg::Disconnected { ip } => {
                self.throttle.dec_con(&ip, &time);
            }
            WsAppMsg::Stop => {
                return Ok(true);
            }
            WsAppMsg::AddListener {
                con_id: connection_key,
                con_tx: tx,
                ws_key,
                done_tx,
            } => {
                self.listener_tracker
                    .cons
                    .insert(connection_key, (ws_key, tx));
                let mut cons: Vec<WsIpStat> = Vec::new();
                for (_, con) in self.throttle.ips.iter() {
                    cons.push(con.stats.clone());
                }
                let send_result = done_tx.send(cons).map_err(|_| WsMsgErr::ListenerDoneTx);
                if let Err(err) = send_result {
                    debug!("ws add listener err: {}", err);
                }
            }
            WsAppMsg::RemoveListener {
                con_id: connection_key,
                // tx,
                // ws_key,
            } => {
                self.listener_tracker.cons.remove(&connection_key);
            }
            WsAppMsg::Ban { ip, date: until, reason } => {
                self.throttle.ban(&ip, reason, until)?;
                debug!("ip {} is banned: {:#?}", ip, &self.throttle.ips); 
                
            }
            // WsAppMsg::UnBan { ip } => {
            //     self.throttle.unban(&ip);
            // }
        }
        Ok(false)
    }
}

impl GetUserAddrMiddleware for ProdUserAddrMiddleware {
    async fn get_addr(&self, addr: SocketAddr) -> SocketAddr {
        addr
    }
}

#[derive(Error, Debug)]
pub enum WsOnConErr {
    #[error("Con tracker err: {0}")]
    ConTracker(#[from] ConTrackerErr),

    #[error("Send error: {0}")]
    IpMangerSend(#[from] tokio::sync::mpsc::error::SendError<IpManagerMsg>),
}

#[derive(Error, Debug)]
pub enum WsMsgErr {
    #[error("failed to send done_tx msg back")]
    ListenerDoneTx,

    #[error("MainGallery error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Send error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),

    #[error("Send error: {0}")]
    IpSend(#[from] tokio::sync::broadcast::error::SendError<IpConMsg>),

    #[error("Send error: {0}")]
    ConnectionSend(#[from] tokio::sync::mpsc::error::SendError<ConMsg>),

    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),
}
