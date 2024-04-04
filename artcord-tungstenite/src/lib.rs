use std::collections::HashMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use artcord_leptos_web_sockets::WsPackage;
use artcord_leptos_web_sockets::WsRouteKey;
use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_client_msg::WsPath;
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::prod_server_msg::ServerMsg;
use artcord_state::util::time::time_is_past;
use artcord_state::util::time::time_passed_days;
use chrono::DateTime;
use chrono::Month;
use chrono::Months;
use chrono::TimeDelta;
use chrono::Utc;
use futures::pin_mut;
use futures::FutureExt;
use futures::TryStreamExt;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::task;

use cfg_if::cfg_if;
use futures::future;
use futures::SinkExt;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::debug;
use tracing::instrument;
use tracing::Instrument;
use tracing::{error, trace};
use ws_route::ws_admin_throttle::WsHandleAdminThrottleError;
use ws_route::ws_main_gallery::WsHandleMainGalleryError;
use ws_route::ws_statistics::WsStatisticsError;
use ws_route::ws_user::WsHandleUserError;
use ws_route::ws_user_gallery::WsHandleUserGalleryError;

use crate::ws_route::ws_admin_throttle::ws_hadnle_admin_throttle;
use crate::ws_route::ws_main_gallery::ws_handle_main_gallery;
use crate::ws_route::ws_statistics;
use crate::ws_route::ws_statistics::ws_statistics;
use crate::ws_route::ws_user::ws_handle_user;
use crate::ws_route::ws_user_gallery::ws_handle_user_gallery;

pub mod ws_route;

const WS_LIMIT_MAX_CONNECTIONS: u64 = 10;
const WS_LIMIT_MAX_RED_FLAGS: u64 = 2;
const WS_EXPIRE_RED_FLAGS_DAYS: u64 = 30;
const WS_BAN_UNTIL_DAYS: u64 = 30;
//const WS_LIMIT_THROTTLE: u64 = 10;

pub struct ThrottleStats {
    ws_connection_count: RwLock<u64>,
    ws_path_count: RwLock<HashMap<WsPath, (u64, Instant)>>,
    ws_red_flag: RwLock<Option<(u64, DateTime<Utc>)>>,
    ws_banned_until: RwLock<Option<DateTime<Utc>>>,
    // ws_proccesing: RwLock<bool>,
    // ws_path_interval: RwLock<DateTime<chrono::Utc>>,
    //ws_last_connection: RwLock<u64>,
}

impl ThrottleStats {
    pub fn new() -> Self {
        Self {
            ws_connection_count: RwLock::new(1),
            ws_path_count: RwLock::new(HashMap::new()),
            ws_red_flag: RwLock::new(None),
            ws_banned_until: RwLock::new(None),
            //   ws_path_interval: RwLock::new(Utc::now())
        }
    }

    pub async fn maybe_sleep(&self, ws_path: &WsPath) {
        let mut ws_path_count_guard = self.ws_path_count.write().await;
        // let (count, interval) = ws_path_count.entry(ws_path).or_insert((1, Instant::now()));
        let ws_path_count = ws_path_count_guard.get_mut(ws_path);
        if let Some((count, interval)) = ws_path_count {
            let (count_limit, interval_limit) = ws_path.get_throttle();
            let elapsed = interval.elapsed();
            if elapsed > interval_limit {
                trace!("throttle: reset");
                *count = 0;
                *interval = Instant::now();
            } else if *count > count_limit {
                let left = interval_limit.checked_sub(elapsed);
                if let Some(left) = left {
                    trace!("throttle: sleeping for: {:?}", &left);
                    sleep(left).await;
                } else {
                    error!("throttle: failed to get left time");
                    sleep(interval_limit).await;
                }
                *count = 0;
                *interval = Instant::now();
                trace!("throttle: sleep completed");
            } else {
                trace!("throttle: all good: state: {} {:?}", &count, &elapsed);
                *count += 1;
            }
        } else {
            let new_ws_path_count = (1_u64, Instant::now());
            ws_path_count_guard.insert(ws_path.clone(), new_ws_path_count);
        }
    }

    // pub async fn maybe_connect_to_ws() {
    //         if let Some(user_throttle_stats) = user_throttle_stats {
    //             trace!("ws({}): throttle: stats exist", &ws_addr);
    //             let count = *user_throttle_stats.ws_connection_count.read().await;
    //
    //             // let (time, count) = *throttle.read().await;
    //
    //             // let throttle = match throttle {
    //             //     Ok(result) => result,
    //             //     Err(err) => {
    //             //         error!("ws({}): lock error: {}", &ws_addr, err);
    //             //         continue;
    //             //     }
    //             // };
    //
    //             // (time, count)
    //             trace!(
    //                 "ws({}): throttle: {} > {}",
    //                 &ws_addr,
    //                 count,
    //                 WS_LIMIT_MAX_CONNECTIONS
    //             );
    //             if count > WS_LIMIT_MAX_CONNECTIONS {
    //                 trace!("ws({}): connection limit reached: {}", &ws_addr, count);
    //                 continue;
    //             }
    //             *user_throttle_stats.ws_connection_count.write().await += 1;
    //             trace!(
    //                 "ws({}): throttle: incremented to: {}",
    //                 &ws_addr,
    //                 *user_throttle_stats.ws_connection_count.read().await
    //             );
    //             user_throttle_stats.clone()
    //         } else {
    //             trace!("ws({}): throttle: created new", &ws_addr);
    //             let user_throttle_stats = Arc::new(ThrottleStats::new());
    //             throttle.insert(ip, user_throttle_stats.clone());
    //             user_throttle_stats
    //         }
    // }

    pub async fn is_banned(&self) -> bool {
        let is_baned = self
            .ws_banned_until
            .read()
            .await
            .map(|until| !time_is_past(until))
            .unwrap_or_else(|| {
                trace!("throttle: ban check: entry doesnt exist");
                false
            });
        trace!(
            "throttle: is banned: {}, state: {:#?}",
            is_baned,
            &*self.ws_banned_until.read().await
        );

        is_baned
    }

    pub async fn maybe_ban(&self) {
        let red_flag = *self.ws_red_flag.read().await;
        if let Some((count, last_modified)) = red_flag {
            if time_passed_days(last_modified, WS_EXPIRE_RED_FLAGS_DAYS) {
                let red_flag = &mut *self.ws_red_flag.write().await;
                trace!("throttle: ws_red_flag: {:?} to None", red_flag,);
                *red_flag = None;
            } else if count > WS_LIMIT_MAX_RED_FLAGS {
                let now = Utc::now();
                let ban = self
                    .ws_banned_until
                    .read()
                    .await
                    .clone()
                    .map(|until| now > until)
                    .unwrap_or(true);

                if ban {
                    let new_date = now + chrono::Days::new(WS_BAN_UNTIL_DAYS);
                    trace!("throttle: banned until: {}", &new_date,);

                    *self.ws_banned_until.write().await = Some(new_date);
                    debug!("IM HEREEEEEEEEEEEEEEEEEEEEEEEEEEEEEE");
                } else {
                    trace!("throttle: is already banned");
                }
                // if let Some(banned_until) = banned_until {
                //     // banned_until.
                // } else {
                //     *banned_until = Some(Utc::now() + Months::new(1));
                // }
            } else {
                let red_flag = &mut *self.ws_red_flag.write().await;
                if let Some((count, last_modified)) = red_flag {
                    let new_date = Utc::now();
                    trace!(
                        "throttle: ws_red_flag: ({}, {}) to ({}, {})",
                        count,
                        last_modified,
                        *count + 1,
                        new_date
                    );
                    *count += 1;
                    *last_modified = new_date;
                } else {
                    error!("throttle: failed to get ws_red_flag");
                }
            }
        } else {
            let new_red_flag = Some((1, Utc::now()));
            trace!("throttle: new ws_red_flag created: {:?}", &new_red_flag);
            *self.ws_red_flag.write().await = new_red_flag;
        }
    }
}

pub struct Throttle {
    pub ips: HashMap<IpAddr, Arc<ThrottleStats>>,
}

impl Throttle {
    pub fn new() -> Self {
        Self {
            ips: HashMap::new(),
        }
    }

    pub async fn maybe_connect_to_ws(&mut self, ip: IpAddr) -> Option<Arc<ThrottleStats>> {
        let user_throttle_stats = self.ips.get(&ip).cloned();
        let Some(user_throttle_stats) = user_throttle_stats else {
            trace!("ws({}): throttle: created new", &ip);
            let user_throttle_stats = Arc::new(ThrottleStats::new());
            self.ips.insert(ip, user_throttle_stats.clone());
            return Some(user_throttle_stats);
        };
        if user_throttle_stats.is_banned().await {
            trace!("ws({}): throttle: is banned", &ip);
            return None;
        }
        trace!("ws({}): throttle: stats exist", &ip);
        let count = *user_throttle_stats.ws_connection_count.read().await;

        // let (time, count) = *throttle.read().await;

        // let throttle = match throttle {
        //     Ok(result) => result,
        //     Err(err) => {
        //         error!("ws({}): lock error: {}", &ws_addr, err);
        //         continue;
        //     }
        // };

        // (time, count)
        trace!(
            "ws({}): throttle: {} > {}",
            &ip,
            count,
            WS_LIMIT_MAX_CONNECTIONS
        );
        if count > WS_LIMIT_MAX_CONNECTIONS {
            trace!("ws({}): connection limit reached: {}", &ip, count);
            return None;
        }
        *user_throttle_stats.ws_connection_count.write().await += 1;
        trace!(
            "ws({}): throttle: incremented to: {}",
            &ip,
            *user_throttle_stats.ws_connection_count.read().await
        );
        Some(user_throttle_stats)
    }
}

pub async fn create_websockets(db: Arc<DB>) {
    let ws_addr = String::from("0.0.0.0:3420");
    let try_socket = TcpListener::bind(&ws_addr).await;
    let listener = try_socket.expect("Failed to bind");
    trace!("ws({}): started", &ws_addr);

    //let throttle = Arc::new(RwLock::new(HashMap::<SocketAddr, AtomicU64>::new()));
    let mut throttle = Throttle::new();

    loop {
        let (stream, user_addr) = match listener.accept().await {
            Ok(result) => result,
            Err(err) => {
                trace!("ws({}): error accepting connection: {}", &ws_addr, err);
                continue;
            }
        };
        debug!("HELLO ONE");
        let Some(user_throttle_stats) = throttle.maybe_connect_to_ws(user_addr.ip()).await else {
            debug!("HELLO TWO");
            continue;
        };
        // let ip = user_addr.ip();
        // let user_throttle_stats = throttle.get(&ip);
        // let user_throttle_stats: Arc<ThrottleStats> =
        //     if let Some(user_throttle_stats) = user_throttle_stats {
        //         trace!("ws({}): throttle: stats exist", &ws_addr);
        //         let count = *user_throttle_stats.ws_connection_count.read().await;
        //
        //         // let (time, count) = *throttle.read().await;
        //
        //         // let throttle = match throttle {
        //         //     Ok(result) => result,
        //         //     Err(err) => {
        //         //         error!("ws({}): lock error: {}", &ws_addr, err);
        //         //         continue;
        //         //     }
        //         // };
        //
        //         // (time, count)
        //         trace!(
        //             "ws({}): throttle: {} > {}",
        //             &ws_addr,
        //             count,
        //             WS_LIMIT_MAX_CONNECTIONS
        //         );
        //         if count > WS_LIMIT_MAX_CONNECTIONS {
        //             trace!("ws({}): connection limit reached: {}", &ws_addr, count);
        //             continue;
        //         }
        //         *user_throttle_stats.ws_connection_count.write().await += 1;
        //         trace!(
        //             "ws({}): throttle: incremented to: {}",
        //             &ws_addr,
        //             *user_throttle_stats.ws_connection_count.read().await
        //         );
        //         user_throttle_stats.clone()
        //     } else {
        //         trace!("ws({}): throttle: created new", &ws_addr);
        //         let user_throttle_stats = Arc::new(ThrottleStats::new());
        //         throttle.insert(ip, user_throttle_stats.clone());
        //         user_throttle_stats
        //     };
        //
        let ws_connection_count = *user_throttle_stats.ws_connection_count.read().await;
        debug!("con count: {}", ws_connection_count);

        // {
        //     let throttle = throttle.read().await;
        //     let user_con = throttle.get(&user_addr);
        //     if let Some(user_con) = user_con {
        //         let count = user_con.as_ptr().re();
        //     }
        // }

        // debug!("addr: {:?}", addr);

        // let Ok(user_addr) = stream
        //     .peer_addr()
        //     .inspect_err(|err| error!("failed to get peer addr: {}", err))
        // else {
        //     continue;
        // };

        task::spawn(
            accept_connection(user_throttle_stats, user_addr, stream, db.clone()).instrument(
                tracing::trace_span!("ws", "{}-{}", ws_addr, user_addr.to_string()),
            ),
        );
    }

    // Ok(())
}

async fn accept_connection(
    user_throttle_task: Arc<ThrottleStats>,
    addr: SocketAddr,
    stream: TcpStream,
    db: Arc<DB>,
) {
    //trace!("connecting...");

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred.");

    trace!("connected");

    let (send, mut recv) = mpsc::channel::<Message>(10);
    let (mut write, mut read) = ws_stream.split();
    let task_tracker = TaskTracker::new();
    let is_admin_throttle_listener_active: Arc<Mutex<Option<JoinHandle<()>>>> =
        Arc::new(Mutex::new(None));
    let (admin_cancel_send, admin_cancel_recv) = broadcast::channel::<bool>(1);
    // let adming_throttle_listener_is_closed = oneshot::channel();
    // let

    let read = async {
        loop {
            let result = read.next().await;
            let Some(result) = result else {
                trace!("read.next() returned None");
                break;
            };
            match result {
                Ok(client_msg) => {
                    // tokio::spawn(future)

                    debug!("proccesing request count: {}", task_tracker.len());
                    if task_tracker.len() > 2 {
                        debug!("WS REQUEST LIMIT REACHED");
                        user_throttle_task.maybe_ban().await;
                        // *user_throttle_task.ws_red_flag.write().await += 1;
                        // *red_flag += 1;
                        break;
                    }

                    let send = send.clone();
                    let db = db.clone();
                    // handler_tasks.len();
                    let user_throttle_task = user_throttle_task.clone();
                    let is_admin_throttle_listener_active =
                        is_admin_throttle_listener_active.clone();
                    let handle_task = {
                        let task_tracker = task_tracker.clone();
                        let admin_cancel_recv = admin_cancel_recv.resubscribe();
                        let admin_cancel_send = admin_cancel_send.clone();

                        // let admin_throttle_listener_close_token =
                        //     admin_throttle_listener_close_token.clone();
                        async move {
                            let _ = response_handler_beta(
                                user_throttle_task,
                                db,
                                send,
                                client_msg,
                                task_tracker,
                                is_admin_throttle_listener_active,
                                admin_cancel_recv,
                                admin_cancel_send,
                            )
                            .await
                            .inspect_err(|err| error!("reponse handler error: {:#?}", err));
                        }
                    };
                    task_tracker.spawn(handle_task);
                }
                Err(err) => {
                    error!("error receiving message: {}", err);
                }
            }
        }
    };
    // let read = read.try_for_each_concurrent(10, {
    //     |client_msg| {
    //         let send = send.clone();
    //         let db = db.clone();
    //         let user_throttle_task = user_throttle_task.clone();
    //         async move {
    //             cfg_if! {
    //                 if #[cfg(feature = "beta")] {
    //                     let _ = response_handler_beta(user_throttle_task, db, send, client_msg)
    //                     .await
    //                     .inspect_err(|err| error!("reponse handler error: {:#?}", err));
    //                 } else {
    //                     let _ = response_handler(user_throttle_task, db, send, client_msg)
    //                     .await
    //                     .inspect_err(|err| error!("reponse handler error: {:#?}", err));
    //                 }
    //             }
    //
    //             Ok(())
    //         }
    //     }
    // });

    let write = async move {
        while let Some(msg) = recv.recv().await {
            write.send(msg).await.unwrap();
        }
    };

    pin_mut!(read, write);

    select! {
        _ = read => {

        },
        _ = write => {

        }
    }

    task_tracker.close();
    task_tracker.wait().await;
    // future::select(read, write).await;

    //*user_throttle_task.ws_connection_count.write().await -= 1;
    // let can_sub = user_throttle_task.ws_connection_count.write().await.checked_sub(1);

    {
        let mut ws_connection_count = user_throttle_task.ws_connection_count.write().await;
        let can_sub = ws_connection_count.checked_sub(1);
        if let Some(new_ws_connection_count) = can_sub {
            *ws_connection_count = new_ws_connection_count;
        } else {
            error!("throttle: failed to subtract 1");
        }
    }

    trace!(
        "disconnected: {}",
        *user_throttle_task.ws_connection_count.read().await
    );
}

async fn response_handler_beta(
    user_throttle_task: Arc<ThrottleStats>,
    db: Arc<DB>,
    send: mpsc::Sender<Message>,
    client_msg: Message,
    task_tracker: TaskTracker,
    is_admin_throttle_listener_active: Arc<Mutex<Option<JoinHandle<()>>>>,
    admin_cancel_recv: broadcast::Receiver<bool>,
    admin_cancel_send: broadcast::Sender<bool>,
) -> Result<(), WsResponseHandlerError> {
    if let Message::Binary(msgclient_msg) = client_msg {
        let client_msg = ClientMsg::from_bytes(&msgclient_msg)?;

        trace!("received: {:?}", &client_msg);
        let key: WsRouteKey<u128, ProdMsgPermKey> = client_msg.key;
        let data = client_msg.data;
        let ws_path: WsPath = WsPath::from(&data);

        // let start = Instant::now();
        // start.
        //let elapsed = start.elapsed();

        user_throttle_task.maybe_sleep(&ws_path).await;

        // sleep(Duration::from_secs(10)).await;

        // if interval > TimeDelta::from() {

        // }
        // let ws_path_count = {
        //     let ws_path_count = ;
        //     // ws_path_count.en
        //     // //.get(&ws_path).cloned()
        //     // let result = match result {
        //     //     Some(a) => a,
        //     //     None => {
        //     //         let new_path_stats = (1, Utc::now());
        //     //         ws_path_count.insert(k, v)
        //     //         new_path_stats
        //     //     }
        //     // }

        // };
        //let ws_path_count = ws_path_count.get_mut(&ws_path).unwrap_or_else(|| {

        // });
        // let ws_path_count = match ws_path_count.get_mut(&ws_path) {
        //     Some(result) => {

        //     }
        //     None => {

        //     }
        // };
        //let ws_path_interval = &*user_throttle_task.ws_path_interval.read().await;
        //return Ok(());

        let server_msg: Result<ServerMsg, WsResponseHandlerError> = match data {
            ClientMsg::GalleryInit { amount, from } => ws_handle_main_gallery(db, amount, from)
                .await
                .map(ServerMsg::MainGallery)
                .map_err(WsResponseHandlerError::MainGallery),
            ClientMsg::UserGalleryInit {
                amount,
                from,
                user_id,
            } => ws_handle_user_gallery(db, amount, from, user_id)
                .await
                .map(ServerMsg::UserGallery)
                .map_err(WsResponseHandlerError::UserGallery),
            ClientMsg::User { user_id } => ws_handle_user(db, user_id)
                .await
                .map(ServerMsg::User)
                .map_err(WsResponseHandlerError::User),
            ClientMsg::Statistics => ws_statistics(db)
                .await
                .map(ServerMsg::Statistics)
                .map_err(WsResponseHandlerError::Statistics),
            ClientMsg::AdminThrottleListenerToggle(state) => ws_hadnle_admin_throttle(
                db,
                state,
                task_tracker,
                is_admin_throttle_listener_active,
                admin_cancel_recv,
                admin_cancel_send,
            )
            .await
            .map(ServerMsg::AdminThrottle)
            .map_err(WsResponseHandlerError::AdminThrottle),
            _ => Ok(ServerMsg::NotImplemented),
        };

        let server_msg = server_msg
            .inspect_err(|err| {
                error!("reponse error: {:#?}", err);
            })
            .unwrap_or(ServerMsg::Error);

        let server_package = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
            key,
            data: server_msg,
        };

        #[cfg(feature = "development")]
        {
            let mut output = format!("{:?}", &server_package);
            output.truncate(100);
            trace!("sent: {}", output);
        }

        let bytes = ServerMsg::as_bytes(server_package)?;

        let server_msg = Message::binary(bytes);
        send.send(server_msg).await?;
    }
    Ok(())
}

async fn response_handler(
    user_throttle_task: Arc<ThrottleStats>,
    db: Arc<DB>,
    send: mpsc::Sender<Message>,
    client_msg: Message,
) -> Result<(), WsResponseHandlerError> {
    if let Message::Binary(msgclient_msg) = client_msg {
        let client_msg = ClientMsg::from_bytes(&msgclient_msg)?;

        trace!("received: {:?}", &client_msg);
        let key: WsRouteKey<u128, ProdMsgPermKey> = client_msg.key;
        let data = client_msg.data;
        let ws_path: WsPath = WsPath::from(&data);

        // let start = Instant::now();
        // start.
        //let elapsed = start.elapsed();

        {
            let mut ws_path_count = user_throttle_task.ws_path_count.write().await;
            let (count, interval) = ws_path_count.entry(ws_path).or_insert((1, Instant::now()));
            let (count_limit, interval_limit) = ws_path.get_throttle();
            let elapsed = interval.elapsed();
            if elapsed > interval_limit {
                trace!("throttle: reset");
                *count = 0;
                *interval = Instant::now();
            } else if *count > count_limit {
                let left = interval_limit.checked_sub(elapsed);
                if let Some(left) = left {
                    trace!("throttle: sleeping for: {:?}", &left);
                    sleep(left).await;
                } else {
                    error!("throttle: failed to get left time");
                    sleep(interval_limit).await;
                }
                *count = 0;
                *interval = Instant::now();
                trace!("throttle: sleep completed");
            } else {
                trace!("throttle: all good: state: {} {:?}", &count, &elapsed);
                *count += 1;
            }
        }

        // if interval > TimeDelta::from() {

        // }
        // let ws_path_count = {
        //     let ws_path_count = ;
        //     // ws_path_count.en
        //     // //.get(&ws_path).cloned()
        //     // let result = match result {
        //     //     Some(a) => a,
        //     //     None => {
        //     //         let new_path_stats = (1, Utc::now());
        //     //         ws_path_count.insert(k, v)
        //     //         new_path_stats
        //     //     }
        //     // }

        // };
        //let ws_path_count = ws_path_count.get_mut(&ws_path).unwrap_or_else(|| {

        // });
        // let ws_path_count = match ws_path_count.get_mut(&ws_path) {
        //     Some(result) => {

        //     }
        //     None => {

        //     }
        // };
        //let ws_path_interval = &*user_throttle_task.ws_path_interval.read().await;
        //return Ok(());

        let server_msg: Result<ServerMsg, WsResponseHandlerError> = match data {
            ClientMsg::GalleryInit { amount, from } => ws_handle_main_gallery(db, amount, from)
                .await
                .map(ServerMsg::MainGallery)
                .map_err(WsResponseHandlerError::MainGallery),
            ClientMsg::UserGalleryInit {
                amount,
                from,
                user_id,
            } => ws_handle_user_gallery(db, amount, from, user_id)
                .await
                .map(ServerMsg::UserGallery)
                .map_err(WsResponseHandlerError::UserGallery),
            ClientMsg::User { user_id } => ws_handle_user(db, user_id)
                .await
                .map(ServerMsg::User)
                .map_err(WsResponseHandlerError::User),
            _ => Ok(ServerMsg::NotImplemented),
        };

        let server_msg = server_msg
            .inspect_err(|err| {
                error!("reponse error: {:#?}", err);
            })
            .unwrap_or(ServerMsg::Error);

        let server_package = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
            key,
            data: server_msg,
        };

        #[cfg(feature = "development")]
        {
            let mut output = format!("{:?}", &server_package);
            output.truncate(100);
            trace!("sent: {}", output);
        }

        let bytes = ServerMsg::as_bytes(server_package)?;

        let server_msg = Message::binary(bytes);
        send.send(server_msg).await?;
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum WsResponseHandlerError {
    #[error("Statistics error: {0}")]
    AdminThrottle(#[from] WsHandleAdminThrottleError),

    #[error("Statistics error: {0}")]
    Statistics(#[from] WsStatisticsError),

    #[error("MainGallery error: {0}")]
    MainGallery(#[from] WsHandleMainGalleryError),

    #[error("MainGallery error: {0}")]
    UserGallery(#[from] WsHandleUserGalleryError),

    #[error("User error: {0}")]
    User(#[from] WsHandleUserError),

    #[error("MainGallery error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Send error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),
    // tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>>>
    // #[error("Mongodb error: {0}")]
    // MongoDB(#[from] mongodb::error::Error),

    // #[error("Bcrypt error: {0}")]
    // Bcrypt(#[from] bcrypt::BcryptError),

    // #[error("JWT error: {0}")]
    // JWT(#[from] jsonwebtoken::errors::Error),

    // #[error("RwLock error: {0}")]
    // RwLock(String),
}
