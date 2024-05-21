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
use artcord_state::misc::throttle_connection::WsReqStat;
use artcord_state::misc::throttle_threshold::AllowCon;
use artcord_state::misc::throttle_threshold::Threshold;
use artcord_state::misc::throttle_threshold::ThrottleRanged;
use artcord_state::misc::throttle_threshold::ThrottleSimple;
use artcord_state::model::ws_statistics;
use artcord_state::model::ws_statistics::DbWsStat;
use artcord_state::model::ws_statistics::TempConIdType;
use artcord_state::util::time::time_is_past;
use artcord_state::util::time::time_passed_days;
use artcord_state::util::time::TimeMiddleware;
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

use self::con::tracker::ConTracker;
use self::con::tracker::ConTrackerErr;
use self::con::Con;
use self::con::ConMsg;

pub mod con;
pub mod throttle;

pub type GlobalConChannel = (
    broadcast::Sender<GlobalConMsg>,
    broadcast::Receiver<GlobalConMsg>,
);

pub enum WsAppMsg {
    Stop,
    Ban {
        ip: IpAddr,
        until: DateTime<Utc>,
        reason: IpBanReason,
    },
    Disconnected {
        connection_key: TempConIdType,
        ip: IpAddr,
    },
    AddListener {
        connection_key: TempConIdType,
        tx: mpsc::Sender<ConMsg>,
        ws_key: WsRouteKey,
    },
    RemoveListener {
        connection_key: TempConIdType,
        tx: mpsc::Sender<ConMsg>,
        ws_key: WsRouteKey,
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
    ws_tx: mpsc::Sender<WsAppMsg>,
    ws_rx: mpsc::Receiver<WsAppMsg>,
    global_con_tx: broadcast::Sender<GlobalConMsg>,
    global_con_rx: broadcast::Receiver<GlobalConMsg>,
    db: Arc<DB>,
    throttle: WsThrottle,
    con_tracker: ConTracker,
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
        let (ws_tx, ws_rx) = mpsc::channel::<WsAppMsg>(1);
        let (global_con_tx, global_con_rx) = broadcast::channel::<GlobalConMsg>(1000);

        let mut ws = Self {
            tcp_listener,
            ws_task_tracker: TaskTracker::new(),
            root_task_tracker,
            root_cancellation_token,
            ws_addr,
            ws_threshold,
            ws_tx,
            ws_rx,
            global_con_tx,
            global_con_rx,
            db,
            throttle: WsThrottle::new(),
            con_tracker: ConTracker::new(),
            time_middleware,
            threshold_middleware,
            socket_middleware,
        };

        ws.run().await;
    }

    pub async fn run(&mut self) {
        info!("ws started");

        //self.root_cancellation_token.is_cancelled()
        loop {
            select! {
                con = self.tcp_listener.accept() => {
                    trace!("con accepted");
                    self.on_con(con).await;
                },

                ws_msg = self.ws_rx.recv() => {
                    trace!("ws recved msg");
                    let exit = self.on_msg(ws_msg).await;
                    let exit = match exit {
                        Ok(exit) => exit,
                        Err(err) => {
                            error!("ws_app: on_ws_msg error: {}", err);
                            continue;
                        }
                    };
                    if exit {
                        trace!("ws_rx closed");
                        break;
                    }
                },

                _ = self.root_cancellation_token.cancelled() => {
                    trace!("ws canceled");
                    break;
                }
            }
        }
        debug!("ws app exiting...");
        self.ws_task_tracker.close();

        loop {
            select! {
                _ =  self.ws_task_tracker.wait() => {
                    break;
                }
                ws_msg = self.ws_rx.recv() => {
                    let exit = self.on_msg(ws_msg).await;
                    let exit = match exit {
                        Ok(exit) => exit,
                        Err(err) => {
                            error!("ws_app: on_ws_msg error: {}", err);
                            continue;
                        }
                    };
                    if exit {
                        break;
                    }
                }
            }
        }

        debug!("ws app exited.");
    }

    pub async fn on_con(
        &mut self,
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

        let allow: bool = match self.throttle.inc_con(ip, &self.ws_threshold, &time) {
            AllowCon::Allow => {
                if !self.con_tracker.cons.is_empty() {
                    let msg = ServerMsg::WsLiveThrottleCachedConnected { ip };
                    self.con_tracker.send(msg).await?;
                }
                true
            }
            AllowCon::AlreadyBanned => false,
            AllowCon::Blocked => {
                if !self.con_tracker.cons.is_empty() {
                    if let Some((total_amount, amount)) = self.throttle.get_amounts(ip) {
                        let msg = ServerMsg::WsLiveThrottleCachedBlocks {
                            ip,
                            total_blocks: total_amount,
                            blocks: amount,
                        };
                        self.con_tracker.send(msg).await?;
                    } else {
                        error!("ws({}): ip amounts not found for: {}", &self.ws_addr, ip);
                    }
                }
                false
            }
            AllowCon::Unbanned => {
                if !self.con_tracker.cons.is_empty() {
                    let msg = ServerMsg::WsLiveThrottleCachedUnban { ip };
                    self.con_tracker.send(msg).await?;
                }
                true
            }
            AllowCon::Banned((date, reason)) => {
                if !self.con_tracker.cons.is_empty() {
                    let msg = ServerMsg::WsLiveThrottleCachedBanned { ip, date, reason };
                    self.con_tracker.send(msg).await?;
                }
                false
            }
        };
        if !allow {
            debug!("ws({}): dont connect", &self.ws_addr);
            return Ok(());
        };

        self.ws_task_tracker.spawn(
            Con::connect(
                stream,
                self.root_cancellation_token.clone(),
                self.db.clone(),
                self.ws_tx.clone(),
                ip,
                user_addr,
                (self.global_con_tx.clone(), self.global_con_tx.subscribe()),
                self.ws_threshold.ws_req_ban_threshold.clone(),
                self.ws_threshold.ws_req_ban_duration.clone(),
                self.time_middleware.clone(),
                self.threshold_middleware.clone(),
            )
            .instrument(tracing::trace_span!(
                "ws",
                "{}-{}",
                self.ws_addr,
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
            WsAppMsg::Disconnected { connection_key, ip } => {
                self.throttle.dec_con(ip, connection_key);
            }
            WsAppMsg::Stop => {
                return Ok(true);
            }
            WsAppMsg::AddListener {
                connection_key,
                tx,
                ws_key,
            } => {
                self.con_tracker.cons.insert(connection_key, (ws_key, tx));
            }
            WsAppMsg::RemoveListener {
                connection_key,
                tx,
                ws_key,
            } => {
                self.con_tracker.cons.remove(&connection_key);
            }
            WsAppMsg::Ban { ip, until, reason } => {
                self.throttle.on_ban(&ip, reason, until);
            }
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
}

#[derive(Error, Debug)]
pub enum WsMsgErr {
    #[error("MainGallery error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Send error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),

    #[error("Send error: {0}")]
    ConnectionSend(#[from] tokio::sync::mpsc::error::SendError<ConMsg>),

    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),
}
