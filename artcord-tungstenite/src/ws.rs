use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use artcord_leptos_web_sockets::WsRouteKey;
use artcord_mongodb::database::model::ws_ip_manager;
use artcord_mongodb::database::DBError;
use artcord_mongodb::database::DB;
use artcord_state::global;
use chrono::DateTime;
use chrono::Month;
use chrono::Months;
use chrono::TimeDelta;
use chrono::Utc;
use con::IpConMsg;
use futures::pin_mut;
use futures::stream::SplitSink;
use futures::stream::SplitStream;
use futures::FutureExt;
use futures::TryStreamExt;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::join;
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

use self::con::throttle_stats_listener_tracker::ConTrackerErr;
use self::con::throttle_stats_listener_tracker::ThrottleStatsListenerTracker;
use self::con::Con;
use self::con::ConMsg;

pub mod con;

pub type GlobalConChannel = (
    broadcast::Sender<GlobalConMsg>,
    broadcast::Receiver<GlobalConMsg>,
);

pub trait GetUserAddrMiddleware {
    fn get_addr(&self, addr: SocketAddr) -> impl std::future::Future<Output = SocketAddr> + Send;
}

#[derive(Debug)]
pub enum WsAppMsg {
    Close,
    Ban {
        ip: IpAddr,
        date: DateTime<Utc>,
        reason: global::IpBanReason,
    },
    // UnBan {
    //     ip: IpAddr,
    // },
    Disconnected {
        //  connection_key: TempConIdType,
        ip: IpAddr,
    },
    AddListener {
        con_id: global::TempConIdType,
        con_tx: mpsc::Sender<ConMsg>,
        ws_key: WsRouteKey,
        done_tx: oneshot::Sender<Vec<global::ConnectedWsIp>>,
    },
    RemoveListener {
        con_id: global::TempConIdType,
        // tx: mpsc::Sender<ConMsg>,
        // ws_key: WsRouteKey,
    },
    // Inc {
    //     ip: IpAddr,
    //     path: ClientPathType,
    // },
}

#[derive(Debug)]
pub enum IpManagerMsg {
    CheckThrottle {
        path: usize,
        block_threshold: global::Threshold,
        allow_tx: oneshot::Sender<global::AllowCon>,
    },
    Unban,
    Close,
    // IncThrottle {
    //     author_id: TempConIdType,
    //     path: usize,
    //     block_threshold: Threshold,
    // },
    // AddIpStatListener {
    //     msg_author: TempConIdType,
    //     con_id: TempConIdType,
    //     con_tx: mpsc::Sender<ConMsg>,
    //     current_state_tx: mpsc::Sender<HashMap<ClientPathType, WsReqStat>>,
    // },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ProdUserAddrMiddleware;

pub struct Ws<
    TimeMiddlewareType: global::TimeMiddleware + Clone + Sync + Send + 'static,
    ThresholdMiddlewareType: global::ClientThresholdMiddleware + Send + Clone + Sync + 'static,
    SocketAddrMiddlewareType: GetUserAddrMiddleware + Send + Sync + Clone + 'static,
> {
    tcp_listener: TcpListener,
    task_tracker_ws_ip_manager: TaskTracker,
    task_tracker_ws_con: TaskTracker,
    task_tracker_root: TaskTracker,
    cancellation_token_root: CancellationToken,
    cancellation_token_ip_manager: CancellationToken,
    cancellation_token_user_con: CancellationToken,
    ws_addr: String,
    ws_threshold: global::DefaultThreshold,
    // ws_tx: mpsc::Sender<WsAppMsg>,
    // ws_rx: mpsc::Receiver<WsAppMsg>,
    global_con_tx: broadcast::Sender<GlobalConMsg>,
    global_con_rx: broadcast::Receiver<GlobalConMsg>,
    db: Arc<DB>,
    ips: HashMap<IpAddr, WsIp>,
    listener_tracker: ThrottleStatsListenerTracker,
    time_middleware: TimeMiddlewareType,
    threshold_middleware: ThresholdMiddlewareType,
    socket_middleware: SocketAddrMiddlewareType,
}

#[derive(Debug)]
pub struct WsIpManagerTask<
    TimeMiddlewareType: global::TimeMiddleware + Clone + Sync + Send + 'static,
> {
    //id: Option<uuid::Uuid>,
    ip: IpAddr,
    db: Arc<DB>,
    ip_req_stat: HashMap<global::ClientPathType, global::WsConReqStat>,
    banned_until: Option<(DateTime<Utc>, global::IpBanReason)>,
    cancelation_token: CancellationToken,
    time_middleware: TimeMiddlewareType,
    ban_threshold: global::Threshold,
    ban_duration: TimeDelta,
    data_sync_rx: mpsc::Receiver<IpManagerMsg>,
}

#[derive(Debug)]
pub struct WsIp {
    pub current_con_count: u64,
    pub total_allow_amount: u64,
    pub total_block_amount: u64,
    pub total_banned_amount: u64,
    pub total_already_banned_amount: u64,
    pub banned_until: Option<(DateTime<Utc>, global::IpBanReason)>,
    pub con_count_tracker: global::ThresholdTracker,
    pub con_flicker_tracker: global::ThresholdTracker,
    // pub con_throttle: global::ThrottleRanged,
    // pub con_flicker_throttle: global::ThrottleSimple,
    pub ip_con_tx: broadcast::Sender<IpConMsg>,
    pub ip_con_rx: broadcast::Receiver<IpConMsg>,
    pub ip_manager_tx: mpsc::Sender<IpManagerMsg>,
    pub ip_manager_task: Option<JoinHandle<()>>,
}

impl<
        TimeMiddlewareType: global::TimeMiddleware + Clone + Sync + Send + 'static,
        ThresholdMiddlewareType: global::ClientThresholdMiddleware + Send + Clone + Sync + 'static,
        SocketAddrMiddlewareType: GetUserAddrMiddleware + Send + Sync + Clone + 'static,
    > Ws<TimeMiddlewareType, ThresholdMiddlewareType, SocketAddrMiddlewareType>
{
    pub async fn create(
        root_task_tracker: TaskTracker,
        root_cancellation_token: CancellationToken,
        ws_addr: String,
        ws_threshold: global::DefaultThreshold,
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
            task_tracker_ws_ip_manager: TaskTracker::new(),
            task_tracker_ws_con: TaskTracker::new(),
            task_tracker_root: root_task_tracker,
            cancellation_token_root: root_cancellation_token,
            cancellation_token_ip_manager: CancellationToken::new(),
            cancellation_token_user_con: CancellationToken::new(),
            ws_addr,
            ws_threshold,
            // ws_tx,
            // ws_rx,
            global_con_tx,
            global_con_rx,
            db,
            ips: HashMap::new(),
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

                _ = self.cancellation_token_root.cancelled() => {
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
        self.cancellation_token_user_con.cancel();
        self.task_tracker_ws_con.close();

        loop {
            select! {
                _ = self.task_tracker_ws_con.wait() => {
                    trace!("ws app exiting... all tasks closed");
                    break;
                }
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

        self.cancellation_token_ip_manager.cancel();
        self.task_tracker_ws_ip_manager.close();
        self.task_tracker_ws_ip_manager.wait().await;

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

        let ws_ip = match self.ips.get_mut(&ip) {
            Some(ip) => ip,
            None => {
                let ws_ip = match self.db.ws_ip_find_one_by_ip(ip).await? {
                    Some(saved_ws_ip) => {
                        debug!("loaded from db ws_ip with flicker at: {}", saved_ws_ip.con_flicker_tracker.amount);
                        WsIp::new(
                        self.db.clone(),
                        ip,
                        saved_ws_ip.total_allow_amount,
                        saved_ws_ip.total_block_amount,
                        saved_ws_ip.total_banned_amount,
                        saved_ws_ip.total_already_banned_amount,
                        self.ws_threshold.ws_req_ban_threshold.clone(),
                        self.ws_threshold.ws_req_ban_duration.clone(),
                        saved_ws_ip.con_count_tracker,
                        saved_ws_ip.con_flicker_tracker,
                        &self.task_tracker_ws_ip_manager,
                        self.cancellation_token_ip_manager.clone(),
                        self.time_middleware.clone(),
                        time,
                    )},
                    None => WsIp::new(
                        self.db.clone(),
                        ip,
                        0,
                        0,
                        0,
                        0,
                        self.ws_threshold.ws_req_ban_threshold.clone(),
                        self.ws_threshold.ws_req_ban_duration.clone(),
                        global::ThresholdTracker::new(time),
                        global::ThresholdTracker::new(time),
                        &self.task_tracker_ws_ip_manager,
                        self.cancellation_token_ip_manager.clone(),
                        self.time_middleware.clone(),
                        time,
                    ),
                };
                self.ips.entry(ip).or_insert(ws_ip)
            }
        };

        let allow: bool = match global::ws_ip_throttle(
            &mut ws_ip.con_flicker_tracker,
            &mut ws_ip.con_count_tracker,
            &mut ws_ip.current_con_count,
            &mut ws_ip.banned_until,
            &self.ws_threshold,
            &time,
        ) {
            global::AllowCon::Allow => {
                ws_ip.total_allow_amount += 1;
                if !self.listener_tracker.cons.is_empty() {
                    let msg = global::ServerMsg::WsLiveStatsConAllowed {
                        ip,
                        total_amount: ws_ip.total_allow_amount,
                    };
                    self.listener_tracker.send(msg).await?
                }
                true
            }
            global::AllowCon::AlreadyBanned => {
                ws_ip.total_already_banned_amount += 1;
                false
            }
            global::AllowCon::Blocked => {
                ws_ip.total_block_amount += 1;
                if !self.listener_tracker.cons.is_empty() {
                    let msg = global::ServerMsg::WsLiveStatsConBlocked {
                        ip,
                        total_amount: ws_ip.total_block_amount,
                    };
                    self.listener_tracker.send(msg).await?;
                }
                false
            }
            global::AllowCon::UnbannedAndBlocked => {
                ws_ip.total_allow_amount += 1;
                if !self.listener_tracker.cons.is_empty() {
                    let msg = global::ServerMsg::WsLiveStatsConBlocked {
                        ip,
                        total_amount: ws_ip.total_block_amount,
                    };
                    self.listener_tracker.send(msg).await?;

                    let msg = global::ServerMsg::WsLiveStatsIpUnbanned { ip };
                    self.listener_tracker.send(msg).await?;
                }
                ws_ip.ip_manager_tx.send(IpManagerMsg::Unban).await?;

                false
            }
            global::AllowCon::UnbannedAndAllow => {
                if !self.listener_tracker.cons.is_empty() {
                    let msg = global::ServerMsg::WsLiveStatsConAllowed {
                        ip,
                        total_amount: ws_ip.total_allow_amount,
                    };
                    self.listener_tracker.send(msg).await?;

                    let msg = global::ServerMsg::WsLiveStatsIpUnbanned { ip };
                    self.listener_tracker.send(msg).await?;
                }
                ws_ip.ip_manager_tx.send(IpManagerMsg::Unban).await?;
                true
            }
            global::AllowCon::Banned((date, reason)) => {
                ws_ip.total_banned_amount += 1;
                if !self.listener_tracker.cons.is_empty() {
                    let msg = global::ServerMsg::WsLiveStatsConBanned {
                        ip,
                        total_amount: ws_ip.total_banned_amount,
                    };
                    self.listener_tracker.send(msg).await?;

                    let msg = global::ServerMsg::WsLiveStatsIpBanned { ip, date, reason };
                    self.listener_tracker.send(msg).await?;
                }
                ws_ip.ip_con_tx.send(IpConMsg::Disconnect)?;
                debug!("ip {} is banned: {:#?}", ip, &ws_ip);
                false
            }
        };

        trace!("throttle result {:?} and INC: {:#?}", allow, &ws_ip);

        if !allow {
            debug!("ws({}): dont connect", &self.ws_addr);
            return Ok(());
        };

        self.task_tracker_ws_con.spawn(
            Con::connect(
                stream,
                self.cancellation_token_user_con.clone(),
                self.db.clone(),
                ws_tx.clone(),
                ip,
                user_addr,
                (self.global_con_tx.clone(), self.global_con_tx.subscribe()),
                ws_ip.ip_con_tx.clone(),
                ws_ip.ip_con_rx.resubscribe(),
                ws_ip.ip_manager_tx.clone(),
                self.ws_threshold.ws_req_ban_threshold.clone(),
                self.ws_threshold.ws_req_ban_duration.clone(),
                self.time_middleware.clone(),
                self.threshold_middleware.clone(),
                self.listener_tracker.clone(),
            )
            .instrument(tracing::trace_span!("con", "{}", user_addr.to_string())),
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
                if let Some(ws_ip) = self.ips.get_mut(&ip) {
                    let new_con_count = ws_ip.current_con_count.checked_sub(1);
                    if let Some(new_con_count) = new_con_count {
                        ws_ip.current_con_count = new_con_count;
                    } else {
                        error!("on disconnect overflow detected");
                        ws_ip.current_con_count = 0;
                    }
                    if ws_ip.current_con_count == 0
                        && global::is_banned(&mut ws_ip.banned_until, &time) != global::IsBanned::Banned
                    {
                        let send_result = ws_ip.ip_manager_tx.send(IpManagerMsg::Close).await;
                        if let Err(err) = send_result {
                            error!("failed to send Close to ip manager of {} err: {}", ip, err);
                        } else {
                            let mut handle = None;
                            std::mem::swap(&mut ws_ip.ip_manager_task, &mut handle);
                            if let Some(handle) = handle {
                                handle.await?;
                            } else {
                                error!("missing ip manager join handle");
                            }
                        }

                        
                        self.db.ws_ip_upsert(
                            &ip,
                            ws_ip.total_allow_amount,
                            ws_ip.total_block_amount,
                            ws_ip.total_banned_amount,
                            ws_ip.total_already_banned_amount,
                            ws_ip.con_count_tracker.clone(),
                            ws_ip.con_flicker_tracker.clone(),
                            ws_ip.banned_until.clone(),
                            &time,
                        ).await?;

                        debug!("saved ws_ip with flicker at: {}", ws_ip.con_flicker_tracker.amount);

                        self.ips.remove(&ip);
                        trace!("disconnected ip {}", &ip);
                    }
                } else {
                    error!("cant disconnect ip that doesnt exist: {}", ip);
                }
            }
            WsAppMsg::Close => {
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
                let mut cons: Vec<global::ConnectedWsIp> = Vec::new();
                for (ip, ws_ip) in self.ips.iter() {
                    cons.push(global::ConnectedWsIp {
                        ip: *ip,
                        banned_until: ws_ip.banned_until.clone(),
                        total_allow_amount: ws_ip.total_allow_amount,
                        total_block_amount: ws_ip.total_block_amount,
                        total_banned_amount: ws_ip.total_banned_amount,
                        total_already_banned_amount: ws_ip.total_already_banned_amount,
                    });
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
            WsAppMsg::Ban {
                ip,
                date,
                reason,
            } => {
                if let Some(ws_ip) = self.ips.get_mut(&ip) {

                    let msg = global::ServerMsg::WsLiveStatsIpBanned { ip, date, reason: reason.clone() };
                    self.listener_tracker.send(msg).await?;

                    ws_ip.banned_until = Some((date, reason));
                    ws_ip.ip_con_tx.send(IpConMsg::Disconnect)?;
                    debug!("ip {} is banned: {:#?}", ip, &self.ips);
                } else {
                    error!("cant ban ip that doesnt exist: {}", ip);
                }
            } // WsAppMsg::UnBan { ip } => {
              //     self.throttle.unban(&ip);
              // }
        }
        Ok(false)
    }
}

impl<TimeMiddlewareType: global::TimeMiddleware + Clone + Sync + Send + 'static>
    WsIpManagerTask<TimeMiddlewareType>
{
    pub async fn manage_ip(
        ip: IpAddr,
        db: Arc<DB>,
        cancelation_token: CancellationToken,
        data_sync_rx: mpsc::Receiver<IpManagerMsg>,
        time_middleware: TimeMiddlewareType,
        ban_threshold: global::Threshold,
        ban_duration: TimeDelta,
    ) {
        let mut task = Self {
            //id: None,
            ip,
            db,
            ip_req_stat: HashMap::new(),
            banned_until: None,
            cancelation_token,
            time_middleware,
            ban_duration,
            ban_threshold,
            data_sync_rx,
        };

        task.run().await;
    }

    pub async fn run(&mut self) {
        trace!("task is running");
        let result = self.on_start().await;
        if let Err(err) = result {
            error!("on_start: {}", err);
        }
        loop {
            select! {
                msg = self.data_sync_rx.recv() => {
                    let Some(msg) = msg else {
                        break;
                    };
                    let exit = self.on_msg(msg).await;
                    if exit {
                        break;
                    }
                }
                _ = self.cancelation_token.cancelled() => {
                    break;
                }
            }
        }
        let result = self.on_close().await;
        if let Err(err) = result {
            error!("on_close: {}", err);
        }
        trace!("task exited");
    }

    async fn on_start(&mut self) -> Result<(), WsIpManagerErr> {
        let ws_ip_manager = self
            .db
            .ws_ip_manager_find_one_by_ip(self.ip.clone())
            .await?;
        let Some(ws_ip_manager) = ws_ip_manager else {
            //self.id = Some(uuid::Uuid::new_v4());
            return Ok(());
        };
        //self.id = Some(ws_ip_manager.id);
        self.ip_req_stat = ws_ip_manager.req_stats;

        Ok(())
    }

    async fn on_msg(&mut self, msg: IpManagerMsg) -> bool {
        trace!("recv: {:#?}", &msg);
        match msg {
            IpManagerMsg::CheckThrottle {
                path,
                block_threshold,
                allow_tx,
            } => {
                let time = self.time_middleware.get_time().await;

                let ip_req_stat = self
                    .ip_req_stat
                    .entry(path)
                    .or_insert_with(|| global::WsConReqStat::new(time));

                let allow = global::double_throttle(
                    &mut ip_req_stat.block_tracker,
                    &mut ip_req_stat.ban_tracker,
                    &block_threshold,
                    &self.ban_threshold,
                    &global::IpBanReason::WsRouteBruteForceDetected,
                    &self.ban_duration,
                    &time,
                    &mut self.banned_until,
                );

                match &allow {
                    global::AllowCon::Allow | global::AllowCon::UnbannedAndAllow => {
                        ip_req_stat.total_allowed_count += 1;
                    }
                    global::AllowCon::Blocked | global::AllowCon::UnbannedAndBlocked => {
                        ip_req_stat.total_blocked_count += 1;
                    }
                    global::AllowCon::Banned(_) => {
                        ip_req_stat.total_banned_count += 1;
                    }
                    global::AllowCon::AlreadyBanned => {
                        ip_req_stat.total_already_banned_count += 1;
                    }
                }

                let send_result = allow_tx.send(allow);
                if send_result.is_err() {
                    error!("failed to send AllowCon");
                }
            }
            IpManagerMsg::Unban => {
                self.banned_until = None;
            }
            IpManagerMsg::Close => {
                return true;
            }
        }
        trace!("recv finished");
        false
    }

    async fn on_close(&mut self) -> Result<(), WsIpManagerErr> {
        let time = self.time_middleware.get_time().await;
        self.db
            .ws_ip_manager_upsert(self.ip, self.ip_req_stat.clone(), &time)
            .await?;
        Ok(())
    }
}

impl WsIp {
    pub fn new<TimeMiddlewareType: global::TimeMiddleware + Clone + Sync + Send + 'static>(
        db: Arc<DB>,
        ip: IpAddr,
        total_allow_amount: u64,
        total_block_amount: u64,
        total_banned_amount: u64,
        total_already_banned_amount: u64,
        req_ban_threshold: global::Threshold,
        req_ban_duration: TimeDelta,
        con_count_tracker: global::ThresholdTracker,
        con_flicker_tracker: global::ThresholdTracker,
        task_tracker: &TaskTracker,
        cancelation_token: CancellationToken,
        time_middleware: TimeMiddlewareType,
        time: DateTime<Utc>,
    ) -> Self {
        let (con_broadcast_tx, con_broadcast_rx) = broadcast::channel(1);
        let (ip_data_sync_tx, ip_data_sync_rx) = mpsc::channel(1);
        let ip_data_sync_task = task_tracker.spawn(
            WsIpManagerTask::manage_ip(
                ip,
                db,
                cancelation_token,
                ip_data_sync_rx,
                time_middleware,
                req_ban_threshold,
                req_ban_duration,
            )
            .instrument(tracing::trace_span!("ip_sync", "{}", ip)),
        );
        let con = Self {
            con_count_tracker,
            con_flicker_tracker,
            current_con_count: 0,
            banned_until: None,
            total_allow_amount,
            total_block_amount,
            total_banned_amount,
            total_already_banned_amount,
            ip_con_tx: con_broadcast_tx,
            ip_con_rx: con_broadcast_rx,
            ip_manager_tx: ip_data_sync_tx,
            ip_manager_task: Some(ip_data_sync_task),
        };
        con
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

    #[error("Send error: {0}")]
    IpSend(#[from] tokio::sync::broadcast::error::SendError<IpConMsg>),

    #[error("db error: {0}")]
    DBError(#[from] DBError),
}

#[derive(Error, Debug)]
pub enum WsIpManagerErr {
    #[error("Mongodb: {0}.")]
    Mongo(#[from] DBError),
}

#[derive(Error, Debug)]
pub enum WsMsgErr {
    #[error("Con tracker err: {0}")]
    ConTracker(#[from] ConTrackerErr),

    #[error("failed to send done_tx msg back")]
    ListenerDoneTx,

    #[error("MainGallery error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("join error: {0}")]
    JoinErr(#[from] tokio::task::JoinError),

    #[error("Send error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),

    #[error("Send error: {0}")]
    IpSend(#[from] tokio::sync::broadcast::error::SendError<IpConMsg>),

    #[error("Send error: {0}")]
    IpManagerSend(#[from] tokio::sync::mpsc::error::SendError<IpManagerMsg>),

    #[error("Send error: {0}")]
    ConnectionSend(#[from] tokio::sync::mpsc::error::SendError<ConMsg>),

    #[error("db error: {0}")]
    DBError(#[from] DBError),
}
