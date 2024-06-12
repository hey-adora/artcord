use artcord_actix::server::create_server;
use artcord_mongodb::database::DB;
use artcord_serenity::create_bot::create_bot;
use artcord_state::global;
use artcord_tungstenite::ws::ProdUserAddrMiddleware;
use artcord_tungstenite::ws::Ws;
use cfg_if::cfg_if;
use chrono::TimeDelta;
use dotenv::dotenv;
use futures::try_join;
use std::{env, sync::Arc};
use tokio::select;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::error;
use tracing::info;
use tracing::trace;
use tracing::Instrument;

#[actix_web::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .unwrap();

    trace!("started!");

    let path = env::current_dir().unwrap();

    let assets_root_dir = env::var("ASSETS_ROOT_DIR").unwrap_or("./target/site".to_string());
    let gallery_root_dir = env::var("GALLERY_ROOT_DIR").unwrap_or("./gallery/".to_string());
    let mongodb_url =
        env::var("MONGO_URL").unwrap_or("mongodb://root:U2L63zXot4n5@localhost:27017".to_string());
    let discord_bot_token = env::var("DISCORD_BOT_TOKEN").ok();
    let discord_bot_default_guild = env::var("DISCORD_DEFAULT_GUILD").ok();

    trace!("current working directory is {}", path.display());
    trace!("current assets directory is {}", assets_root_dir);
    trace!("current gallery directory is {}", gallery_root_dir);

    //let assets_root_dir = Arc::new(assets_root_dir);
    //let gallery_root_dir = Arc::new(gallery_root_dir);
    let db = DB::new("artcord", mongodb_url).await;
    let db = Arc::new(db);
    let time_machine = global::Clock;

    let task_tracker = TaskTracker::new();
    let cancelation_token = CancellationToken::new();

    let web_server = create_server(&gallery_root_dir, &assets_root_dir).await;

    // cfg_if! {
    //     if #[cfg(feature = "development")] {

    //     } else {

    //     }
    // }

    cfg_if! {
        if #[cfg(feature = "development")] {
            let threshold = WsThreshold {
                ws_max_con_threshold: Threshold::new_const(10, TimeDelta::try_minutes(1)),
                ws_max_con_ban_duration: match TimeDelta::try_minutes(1) {
                    Some(delta) => delta,
                    None => panic!("invalid delta"),
                },
                ws_max_con_threshold_range: 5,
                ws_max_con_ban_reason: IpBanReason::WsTooManyReconnections,
                ws_con_flicker_threshold: Threshold::new_const(10, TimeDelta::try_minutes(1)),
                ws_con_flicker_ban_duration: match TimeDelta::try_minutes(1) {
                    Some(delta) => delta,
                    None => panic!("invalid delta"),
                },
                ws_con_flicker_ban_reason: IpBanReason::WsConFlickerDetected,
                ws_req_ban_threshold: Threshold::new_const(10, TimeDelta::try_minutes(1)),
                ws_req_ban_duration: match TimeDelta::try_minutes(1) {
                    Some(delta) => delta,
                    None => panic!("invalid delta"),
                },
            };
        } else {
            let threshold = global::DefaultThreshold {
                ws_max_con_threshold: global::Threshold::new_const(10000, TimeDelta::try_minutes(1)),
                ws_max_con_ban_duration: match TimeDelta::try_days(1) {
                    Some(delta) => delta,
                    None => panic!("invalid delta"),
                },
                ws_max_con_threshold_range: 100,
                ws_max_con_ban_reason: global::IpBanReason::WsTooManyReconnections,
                ws_con_flicker_threshold: global::Threshold::new_const(10000, TimeDelta::try_minutes(1)),
                ws_con_flicker_ban_duration: match TimeDelta::try_days(1) {
                    Some(delta) => delta,
                    None => panic!("invalid delta"),
                },
                ws_con_flicker_ban_reason: global::IpBanReason::WsConFlickerDetected,
                ws_req_ban_threshold: global::Threshold::new_const(10000, TimeDelta::try_minutes(1)),
                ws_req_ban_duration: match TimeDelta::try_days(1) {
                    Some(delta) => delta,
                    None => panic!("invalid delta"),
                },
            };
        }
    }

    let ws_ip = "0.0.0.0:3420".to_string();
    let web_sockets_handle = task_tracker.spawn(
        Ws::create(
            task_tracker.clone(),
            cancelation_token.clone(),
            ws_ip.clone(),
            threshold,
            db.clone(),
            time_machine,
            global::ProdThreshold,
            ProdUserAddrMiddleware,
        )
        .instrument(tracing::trace_span!("ws", "{}", ws_ip,)),
    );

    //aaa
    if let Some(discord_bot_token) = discord_bot_token {
        let mut discord_bot = create_bot(
            db.clone(),
            &discord_bot_token,
            &gallery_root_dir,
            discord_bot_default_guild,
        )
        .await;

        select! {
            _ = discord_bot.start() => {},
            _ = web_sockets_handle => {},
            _ = web_server => {},
            _ = signal::ctrl_c() => {},
        }
    } else {
        error!("DISCORD_BOT_TOKEN in .env is missing, bot will not start.");
        select! {
            _ = web_sockets_handle => {},
            _ = web_server => {},
            _ = signal::ctrl_c() => {},
        }
    }

    info!("exiting...");
    cancelation_token.cancel();
    task_tracker.close();
    task_tracker.wait().await;
}

#[cfg(test)]
mod artcord_tests {
    use std::{
        collections::{HashMap, HashSet},
        env,
        net::{IpAddr, Ipv4Addr},
        str::FromStr,
        sync::Arc,
        time::Duration,
    };

    use crate::create_server;
    use artcord_mongodb::database::DB;
    use artcord_state::global;
    use artcord_tungstenite::ws::{GetUserAddrMiddleware, Ws};
    use chrono::{DateTime, TimeDelta, Utc};
    use futures::{stream::SplitSink, SinkExt, StreamExt};
    use mongodb::{bson::doc, options::ClientOptions};
    use std::net::SocketAddr;
    use thiserror::Error;
    use tokio::sync::mpsc;
    use tokio::sync::oneshot;
    use tokio::{net::TcpStream, sync::Mutex};
    use tokio::{select, time::sleep};
    use tokio_tungstenite::{
        connect_async,
        tungstenite::{protocol::CloseFrame, Message},
        WebSocketStream,
    };
    use tokio_util::{sync::CancellationToken, task::TaskTracker};
    use tracing::{debug, error, info, trace, Instrument, Level};

    // const MONGO_NAME: &'static str = "artcord_test";
    // const MONGO_NAME2: &'static str = "artcord_test2";
    // const MONGO_NAME3: &'static str = "artcord_test3";
    const MONGO_URL: &'static str = "mongodb://root:U2L63zXot4n5@localhost:27017";
    const CON_MAX_AMOUNT: u64 = 5;
    const CON_MAX_BLOCK_AMOUNT: u64 = 10;
    const CON_BLOCK_DURATION: TimeDelta = match TimeDelta::try_minutes(1) {
        Some(delta) => delta,
        None => panic!("invalid delta"),
    };
    const CON_BAN_DURATION: TimeDelta = match TimeDelta::try_minutes(1) {
        Some(delta) => delta,
        None => panic!("invalid delta"),
    };
    const CON_FLICKER_MAX: u64 = 10;
    const CON_FLICKER_BLOCK_DURATION: TimeDelta = match TimeDelta::try_minutes(1) {
        Some(delta) => delta,
        None => panic!("invalid delta"),
    };
    const CON_FLICKER_BAN_DURATION: TimeDelta = match TimeDelta::try_minutes(1) {
        Some(delta) => delta,
        None => panic!("invalid delta"),
    };
    const REQ_MAX_ALLOW: u64 = 5;
    const REQ_ALLOW_DURATION: TimeDelta = match TimeDelta::try_seconds(10) {
        Some(delta) => delta,
        None => panic!("failed to create delta"),
    };
    const REQ_MAX_BLOCK: u64 = 10;
    const REQ_BLOCK_DURATION: TimeDelta = match TimeDelta::try_seconds(10) {
        Some(delta) => delta,
        None => panic!("failed to create delta"),
    };

    const REQ_BAN_DURATION: TimeDelta = match TimeDelta::try_minutes(1) {
        Some(delta) => delta,
        None => panic!("invalid delta"),
    };

    const CLIENT_CONNECTED_SUCCESS: Result<ConnectionMsg, ClientErr> = Ok(ConnectionMsg::Connected);
    const CLIENT_CONNECTED_ERR: Result<ConnectionMsg, ClientErr> = Err(ClientErr::FailedToConnect);

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub struct DebugThreshold;

    struct Connection {
        key: u128,
        ip: IpAddr,
        client_tx: mpsc::Sender<DebugClientMsg>,
        server_rx: mpsc::Receiver<(u128, global::ServerMsg)>,
        connection_rx: mpsc::Receiver<Result<ConnectionMsg, ClientErr>>,
    }

    #[derive(Debug, PartialEq, Clone)]
    enum DebugClientMsg {
        Send((u128, global::ClientMsg)),
        Disconnect,
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    enum ConnectionMsg {
        Connected,
        Disconnected,
    }

    #[derive(Clone, Debug)]
    pub struct TestClock {
        time_tx: mpsc::Sender<oneshot::Sender<DateTime<Utc>>>,
    }

    #[derive(Debug, Clone)]
    pub struct TestUserAddrMiddleware {
        tx: mpsc::Sender<(SocketAddr, oneshot::Sender<SocketAddr>)>,
    }

    impl GetUserAddrMiddleware for TestUserAddrMiddleware {
        async fn get_addr(&self, addr: SocketAddr) -> SocketAddr {
            let (addr_tx, addr_rx) = oneshot::channel();
            self.tx.send((addr, addr_tx)).await.unwrap();
            addr_rx.await.unwrap()
        }
    }

    impl TestUserAddrMiddleware {
        pub fn new() -> (
            mpsc::Receiver<(SocketAddr, oneshot::Sender<SocketAddr>)>,
            Self,
        ) {
            let (tx, rx) = mpsc::channel::<(SocketAddr, oneshot::Sender<SocketAddr>)>(1);

            (rx, Self { tx })
        }
    }

    impl TestClock {
        pub fn new() -> (Self, mpsc::Receiver<oneshot::Sender<DateTime<Utc>>>) {
            let (time_tx, time_rx) = mpsc::channel(1);
            (Self { time_tx }, time_rx)
        }
    }

    impl global::TimeMiddleware for TestClock {
        async fn get_time(&self) -> DateTime<Utc> {
            let (time_tx, time_rx) = oneshot::channel::<DateTime<Utc>>();
            self.time_tx.send(time_tx).await.unwrap();
            time_rx.await.unwrap()
        }
    }

    impl global::ClientThresholdMiddleware for DebugThreshold {
        fn get_threshold(&self, msg: &global::ClientMsg) -> global::Threshold {
            match msg {
                _ => global::Threshold::new(REQ_MAX_ALLOW, REQ_ALLOW_DURATION),
            }
        }
    }

    struct WsTestApp {
        //ips: HashMap<usize, IpAddr>,
        connections: HashMap<usize, Connection>,
        tracker: TaskTracker,
        time: Arc<Mutex<DateTime<Utc>>>,
        cancelation_token: CancellationToken,
        addr_middleware_rx: mpsc::Receiver<(SocketAddr, oneshot::Sender<SocketAddr>)>,
        server_port: usize,
    }

    impl WsTestApp {
        pub async fn new(ws_id: usize, time: DateTime<Utc>) -> Self {
            let mongo_name = format!("artcord_test_{}", ws_id);
            drop_db(mongo_name.clone(), MONGO_URL).await;
            let db = DB::new(mongo_name, MONGO_URL).await;
            let db = Arc::new(db);
            let (time_machine, mut time_rx) = TestClock::new();
            let time: Arc<Mutex<DateTime<Utc>>> = Arc::new(Mutex::new(time));
            let tracker = TaskTracker::new();
            let cancelation_token = CancellationToken::new();
            let threshold = global::DefaultThreshold {
                ws_max_con_threshold: global::Threshold::new(
                    CON_MAX_BLOCK_AMOUNT,
                    CON_BLOCK_DURATION,
                ),
                ws_max_con_ban_duration: CON_BAN_DURATION,
                ws_max_con_threshold_range: CON_MAX_AMOUNT,
                ws_max_con_ban_reason: global::IpBanReason::WsTooManyReconnections,
                ws_con_flicker_threshold: global::Threshold::new(
                    CON_FLICKER_MAX,
                    CON_FLICKER_BLOCK_DURATION,
                ),
                ws_con_flicker_ban_duration: CON_FLICKER_BAN_DURATION,
                ws_con_flicker_ban_reason: global::IpBanReason::WsConFlickerDetected,
                ws_req_ban_threshold: global::Threshold::new(REQ_MAX_BLOCK, REQ_BLOCK_DURATION),
                ws_req_ban_duration: REQ_BAN_DURATION,
            };

            let (addr_middleware_rx, addr_middleware) = TestUserAddrMiddleware::new();

            tracker.spawn({
                let time = time.clone();
                async move {
                    while let Some(time_rx) = time_rx.recv().await {
                        let time = *time.lock().await;
                        debug!("TIME SENT: {}", time);
                        time_rx.send(time).unwrap();
                    }
                }
            });

            let port = 3420 + ws_id;
            let ws_addr = format!("0.0.0.0:{}", port);
            tracker.spawn(
                Ws::create(
                    tracker.clone(),
                    cancelation_token.clone(),
                    ws_addr.clone(),
                    threshold,
                    db.clone(),
                    time_machine,
                    DebugThreshold,
                    addr_middleware,
                )
                .instrument(tracing::trace_span!("ws", "{}", ws_addr)),
            );

            Self {
                //ips: HashMap::new(),
                connections: HashMap::new(),
                cancelation_token,
                tracker,
                time,
                addr_middleware_rx,
                server_port: port,
            }
        }

        #[track_caller]
        async fn create_client(
            &mut self,
            id: usize,
            ip: Ipv4Addr,
            expect: Result<ConnectionMsg, ClientErr>,
        ) -> usize {
            let mut con = Connection::new(
                self.tracker.clone(),
                self.cancelation_token.clone(),
                IpAddr::V4(ip),
                self.server_port,
            );

            let ((mut addr, return_tx)) = self.addr_middleware_rx.recv().await.unwrap();
            let client_2_ip = IpAddr::V4(ip);
            addr.set_ip(client_2_ip);
            return_tx.send(addr).unwrap();

            assert_eq!(con.recv_command().await, expect);

            self.connections.insert(id, con);
            id
        }

        async fn close_client(&mut self, client_id: usize) {
            let Some(con) = self.connections.get_mut(&client_id) else {
                panic!("missing con: {}", client_id);
            };
            con.command(DebugClientMsg::Disconnect).await;
            let msg = con.recv_command().await.unwrap();
            assert_eq!(msg, ConnectionMsg::Disconnected);
        }

        async fn set_time(&self, callback: impl Fn(&mut DateTime<Utc>)) {
            let time = &mut *self.time.lock().await;
            callback(time);
        }

        async fn send(
            &self,
            client_id: usize,
            msg: global::ClientMsg,
        ) -> std::result::Result<(), tokio::sync::mpsc::error::SendError<DebugClientMsg>> {
            let Some(con) = self.connections.get(&client_id) else {
                panic!("missing con: {}", client_id);
            };
            con.client_tx.send(DebugClientMsg::Send((0, msg))).await
        }

        async fn send_test_msg_once(&mut self, send_client_id: usize) {
            self.send_custom_msg_once(send_client_id, global::ClientMsg::Logout)
                .await;
        }

        async fn send_custom_msg_once(&mut self, send_client_id: usize, msg: global::ClientMsg) {
            self.send(send_client_id, msg).await.unwrap()
        }

        async fn fail_to_send_test_msg_once(&mut self, send_client_id: usize) {
            let r: Result<(), mpsc::error::SendError<DebugClientMsg>> =
                self.send(send_client_id, global::ClientMsg::Logout).await;
            assert!(r.is_err())
        }

        async fn set_time_unblocked(&self) {
            let time = &mut (*self.time.lock().await);
            *time += REQ_ALLOW_DURATION;
        }

        async fn add_time(&self, add_this_time: TimeDelta) {
            let time = &mut (*self.time.lock().await);
            *time += add_this_time;
        }

        async fn recv(&mut self, client_id: usize) -> global::ServerMsg {
            let Some(con) = self.connections.get_mut(&client_id) else {
                panic!("missing con: {}", client_id);
            };

            let msg = con.recv().await;
            debug!("recv: {:#?}", msg);
            msg
        }

        async fn recv_command_disconnected(&mut self, client_id: usize) {
            let Some(con) = self.connections.get_mut(&client_id) else {
                panic!("missing con: {}", client_id);
            };

            let msg = con.recv_command().await;

            assert_eq!(msg, Ok(ConnectionMsg::Disconnected));
        }

        async fn recv_command_boom(&mut self, client_id: usize) {
            let Some(con) = self.connections.get_mut(&client_id) else {
                panic!("missing con: {}", client_id);
            };

            let msg = con.recv_command().await;

            assert_eq!(msg, Err(ClientErr::Boom));
        }

        async fn recv_command(&mut self, client_id: usize) -> Result<ConnectionMsg, ClientErr> {
            let Some(con) = self.connections.get_mut(&client_id) else {
                panic!("missing con: {}", client_id);
            };

            let msg = con.recv_command().await;
            debug!("recv: {:#?}", msg);
            msg
        }

        async fn send_live_stats_on(&self, client_id: usize) {
            self.send(client_id, global::ClientMsg::LiveWsStats(true))
                .await;
        }

        async fn send_live_stats_off(&self, client_id: usize) {
            self.send(client_id, global::ClientMsg::LiveWsStats(false))
                .await;
        }

        async fn recv_connections(&mut self, client_id: usize) {
            let msg = self.recv(client_id).await;
            assert!(matches!(msg, global::ServerMsg::WsLiveStatsIpCons(_)));
        }

        async fn recv_connected(&mut self, client_id: usize) {
            //let mut received: Vec<IpAddr> = Vec::new();
            let ips: Vec<IpAddr> = self.connections.values().map(|con| con.ip).collect();
            trace!("current ips set: {:#?}", ips);
            let good = |msg: global::ServerMsg| match msg {
                global::ServerMsg::WsLiveStatsConnected {
                    ip,
                    socket_addr,
                    con_id,
                    banned_until,
                    req_stats,
                } => {
                    if ips.iter().any(|known_ip| ip == *known_ip)
                    // && !received.iter().any(|ip| stat.ip == *ip)
                    {
                        //received.push(stat.ip);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            for _ in 0..ips.len() {
                let msg = self.recv(client_id).await;
                assert!(good(msg));
            }
        }

        async fn recv_connected_one(&mut self, client_id: usize, target: usize) {
            let targer_ip = self.connections.get(&target).map(|con| con.ip).unwrap();
            let msg = self.recv(client_id).await;
            assert!(match msg {
                global::ServerMsg::WsLiveStatsConnected {
                    ip,
                    socket_addr,
                    con_id,
                    banned_until,
                    req_stats,
                } => ip == targer_ip,
                _ => false,
            })
        }

        async fn recv_disconnected_one(&mut self, client_id: usize) {
            let msg = self.recv(client_id).await;
            assert!(match msg {
                global::ServerMsg::WsLiveStatsDisconnected { con_id } => true,
                _ => false,
            })
        }

        async fn recv_req_allow(&mut self, client_id: usize) {
            let msg = self.recv(client_id).await;
            assert!(match msg {
                global::ServerMsg::WsLiveStatsReqAllowed {
                    con_id,
                    path,
                    total_amount,
                } => true,
                _ => false,
            })
        }

        async fn recv_req_block(&mut self, client_id: usize) {
            let msg = self.recv(client_id).await;
            assert!(match msg {
                global::ServerMsg::WsLiveStatsReqBlocked {
                    con_id,
                    path,
                    total_amount,
                } => true,
                _ => false,
            })
        }

        async fn recv_req_ban(&mut self, client_id: usize) {
            let msg = self.recv(client_id).await;
            assert!(match msg {
                global::ServerMsg::WsLiveStatsReqBanned {
                    con_id,
                    path,
                    total_amount,
                } => true,
                _ => false,
            })
        }

        async fn recv_con_allow(&mut self, client_id: usize, target: usize) {
            let c_ip = self.connections.get(&target).map(|con| con.ip).unwrap();
            let msg = self.recv(client_id).await;
            assert!(match msg {
                global::ServerMsg::WsLiveStatsConAllowed { ip, total_amount } => ip == c_ip,
                _ => false,
            })
        }

        async fn recv_con_block(&mut self, client_id: usize, target: usize) {
            let c_ip = self.connections.get(&target).map(|con| con.ip).unwrap();
            let msg = self.recv(client_id).await;
            assert!(match msg {
                global::ServerMsg::WsLiveStatsConBlocked { ip, total_amount } => ip == c_ip,
                _ => false,
            })
        }

        async fn recv_con_banned(&mut self, client_id: usize, target: usize) {
            let c_ip = self.connections.get(&target).map(|con| con.ip).unwrap();
            let msg = self.recv(client_id).await;
            assert!(match msg {
                global::ServerMsg::WsLiveStatsConBanned { ip, total_amount } => ip == c_ip,
                _ => false,
            })
        }

        async fn recv_ip_banned(
            &mut self,
            client_id: usize,
            target: usize,
            target_reason: global::IpBanReason,
        ) {
            let c_ip = self.connections.get(&target).map(|con| con.ip).unwrap();
            let msg = self.recv(client_id).await;
            assert!(match msg {
                global::ServerMsg::WsLiveStatsIpBanned { ip, date, reason } =>
                    ip == c_ip && reason == target_reason,
                _ => false,
            })
        }

        async fn recv_ip_unban(&mut self, client_id: usize, target: usize) {
            let c_ip = self.connections.get(&target).map(|con| con.ip).unwrap();
            let msg = self.recv(client_id).await;
            assert!(match msg {
                global::ServerMsg::WsLiveStatsIpUnbanned { ip } => ip == c_ip,
                _ => false,
            })
        }

        async fn close(&self) {
            info!("exiting...");
            self.cancelation_token.cancel();
            self.tracker.close();
            self.tracker.wait().await;
        }
    }

    impl Connection {
        pub fn new(
            task_tracker: TaskTracker,
            cancellation_token: CancellationToken,
            client_ip: IpAddr,
            server_port: usize,
        ) -> Self {
            let (connection_tx, connection_rx) =
                mpsc::channel::<Result<ConnectionMsg, ClientErr>>(100);
            let (client_tx, mut client_rx) = mpsc::channel::<DebugClientMsg>(100);
            let (server_tx, server_rx) = mpsc::channel::<(u128, global::ServerMsg)>(100);

            task_tracker.spawn(async move {
                let url = url::Url::parse(&format!("ws://localhost:{}", server_port)).unwrap();
                let con = connect_async(url).await;
                let Ok((ws_stream, res)) = con else {
                    let _ = connection_tx.send(Err(ClientErr::FailedToConnect)).await;
                    return;
                };
                let _ = connection_tx.send(Ok(ConnectionMsg::Connected)).await;
                let (mut write, mut read) = ws_stream.split();

                loop {
                    select! {
                        msg = client_rx.recv() => {
                            let Some(msg) = msg else {
                                break;
                            };
                            let exit = on_client_recv(&mut write, msg).await;
                            if exit {
                                break;
                            }
                        }
                        msg = read.next() => {
                            let exit = on_read(msg, &server_tx).await;
                            if exit {
                                break;
                            }
                        }
                        _ = cancellation_token.cancelled() => {
                            break;
                        }
                    }
                }
                let close_frame = CloseFrame {
                    code:
                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
                    reason: std::borrow::Cow::Borrowed("boom"),
                };
                let _ = write.send(Message::Close(Some(close_frame))).await;
                let _ = connection_tx.send(Ok(ConnectionMsg::Disconnected)).await;
                debug!("client exited.");
            });

            Self {
                key: 0,
                ip: client_ip,
                client_tx,
                server_rx,
                connection_rx,
            }
        }

        pub async fn command(&self, msg: DebugClientMsg) {
            self.client_tx.send(msg).await.unwrap();
        }

        pub async fn recv_command(&mut self) -> Result<ConnectionMsg, ClientErr> {
            self.connection_rx
                .recv()
                .await
                .unwrap_or(Err(ClientErr::Boom))
        }

        pub async fn send(&mut self, msg: global::ClientMsg) {
            self.client_tx
                .send(DebugClientMsg::Send((self.key, msg)))
                .await
                .unwrap();
        }

        pub async fn recv(&mut self) -> global::ServerMsg {
            let (_, msg) = self.server_rx.recv().await.unwrap();
            msg
        }
    }

    #[tokio::test]
    async fn throttle_connect_and_disconnect() {
        init_tracer();

        let time = Utc::now();
        let mut ws_test_app = WsTestApp::new(1, time).await;

        let client1 = ws_test_app
            .create_client(1, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;
        let client2 = ws_test_app
            .create_client(2, Ipv4Addr::new(0, 0, 0, 2), CLIENT_CONNECTED_SUCCESS)
            .await;
        ws_test_app.send_live_stats_on(client2).await;
        ws_test_app.recv_connections(client2).await;
        ws_test_app.recv_connected(client2).await;

        ws_test_app.close_client(client1).await;
        ws_test_app.recv_disconnected_one(client2).await;

        let client1 = ws_test_app
            .create_client(1, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.recv_con_allow(client2, client1).await;
        ws_test_app.recv_connected_one(client2, client1).await;

        ws_test_app.close().await;
    }

    #[tokio::test]
    async fn throttle_req_ban() {
        init_tracer();

        let time = Utc::now();
        let mut ws_test_app = WsTestApp::new(2, time).await;

        let client1 = ws_test_app
            .create_client(1, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;
        let client11 = ws_test_app
            .create_client(11, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;
        let client2 = ws_test_app
            .create_client(2, Ipv4Addr::new(0, 0, 0, 2), CLIENT_CONNECTED_SUCCESS)
            .await;
        let client3 = ws_test_app
            .create_client(3, Ipv4Addr::new(0, 0, 0, 3), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.send_live_stats_on(client2).await;
        ws_test_app.recv_connections(client2).await;
        ws_test_app.recv_connected(client2).await;

        info!("point 1");

        ws_test_app.send_test_msg_once(client3).await;
        ws_test_app.recv_req_allow(client2).await;

        info!("point 2");

        for _ in 0..REQ_MAX_ALLOW {
            ws_test_app.send_test_msg_once(client1).await;
            ws_test_app.recv_req_allow(client2).await;
        }
        for _ in 0..REQ_MAX_BLOCK {
            ws_test_app.send_test_msg_once(client1).await;
            ws_test_app.recv_req_block(client2).await;
        }

        info!("point 3");

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_ban(client2).await;
        ws_test_app
            .recv_ip_banned(
                client2,
                client1,
                global::IpBanReason::WsRouteBruteForceDetected,
            )
            .await;

        info!("point 4");

        ws_test_app.recv_disconnected_one(client2).await;
        ws_test_app.recv_disconnected_one(client2).await;

        info!("point 5");

        ws_test_app.fail_to_send_test_msg_once(client11).await;

        info!("point 6");
        //ws_test_app.recv_req_allow(client2).await;

        ws_test_app.send_test_msg_once(client3).await;
        ws_test_app.recv_req_allow(client2).await;

        info!("point 7");

        ws_test_app.send_test_msg_once(client3).await;
        ws_test_app.recv_req_allow(client2).await;

        info!("point 8");

        let client111 = ws_test_app
            .create_client(111, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_ERR)
            .await;

        info!("point 9");

        ws_test_app.recv_command_disconnected(client1).await;
        ws_test_app.recv_command_disconnected(client11).await;

        info!("point 10");

        ws_test_app.recv_command_boom(client111).await;

        info!("point 11");

        ws_test_app.send_test_msg_once(client3).await;
        ws_test_app.recv_req_allow(client2).await;

        info!("point 12");

        ws_test_app.add_time(REQ_BAN_DURATION).await;

        let client1 = ws_test_app
            .create_client(1, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;

        info!("point 13");

        ws_test_app.recv_con_allow(client2, client1).await;
        ws_test_app.recv_ip_unban(client2, client1).await;
        ws_test_app.recv_connected_one(client2, client1).await;

        info!("point 14");

        for _ in 0..REQ_MAX_ALLOW {
            ws_test_app.send_test_msg_once(client3).await;
            ws_test_app.recv_req_allow(client2).await;
        }

        ws_test_app.close_client(client3).await;

        ws_test_app.recv_disconnected_one(client2).await;

        let client3 = ws_test_app
            .create_client(3, Ipv4Addr::new(0, 0, 0, 3), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.recv_con_allow(client2, client3).await;
        ws_test_app.recv_connected_one(client2, client3).await;

        ws_test_app.send_test_msg_once(client3).await;
        ws_test_app.recv_req_block(client2).await;

        info!("point 15");

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;

        info!("point 16");

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;

        info!("point 17");

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;

        info!("point 18");

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;

        info!("point 19");

        ws_test_app.close().await;
    }

    #[tokio::test]
    async fn throttle_too_many_cons_ban() {
        init_tracer();

        let time = Utc::now();
        let mut ws_test_app = WsTestApp::new(3, time).await;

        let client2 = ws_test_app
            .create_client(200, Ipv4Addr::new(0, 0, 0, 2), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.send_live_stats_on(client2).await;
        ws_test_app.recv_connections(client2).await;
        ws_test_app.recv_connected_one(client2, client2).await;

        info!("GET BLOCKED");

        for i in 0..CON_MAX_AMOUNT as usize {
            let client = ws_test_app
                .create_client(i, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
                .await;
            ws_test_app.recv_con_allow(client2, client).await;
            ws_test_app.recv_connected_one(client2, client).await;
        }

        for i in CON_MAX_AMOUNT as usize..CON_MAX_BLOCK_AMOUNT as usize + CON_MAX_AMOUNT as usize {
            let client = ws_test_app
                .create_client(i, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_ERR)
                .await;
            ws_test_app.recv_con_block(client2, client).await;
        }

        info!("GET UNBLOCKED");

        ws_test_app.add_time(CON_BLOCK_DURATION).await;
        for i in 0..CON_MAX_AMOUNT as usize {
            ws_test_app.close_client(i).await;
            ws_test_app.recv_disconnected_one(client2).await;
        }

        info!("GET BLOCKED AGAIN");

        for i in 0..CON_MAX_AMOUNT as usize {
            let client = ws_test_app
                .create_client(i, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
                .await;
            ws_test_app.recv_con_allow(client2, client).await;
            ws_test_app.recv_connected_one(client2, client).await;
        }

        for i in CON_MAX_AMOUNT as usize..CON_MAX_BLOCK_AMOUNT as usize + CON_MAX_AMOUNT as usize {
            let client = ws_test_app
                .create_client(i, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_ERR)
                .await;
            ws_test_app.recv_con_block(client2, client).await;
        }

        info!("GET BANNED");

        let client = ws_test_app
            .create_client(
                CON_MAX_BLOCK_AMOUNT as usize + CON_MAX_AMOUNT as usize,
                Ipv4Addr::new(0, 0, 0, 1),
                CLIENT_CONNECTED_ERR,
            )
            .await;
        ws_test_app.recv_con_banned(client2, client).await;
        ws_test_app
            .recv_ip_banned(client2, client, global::IpBanReason::WsTooManyReconnections)
            .await;

        for _ in 0..CON_MAX_AMOUNT as usize {
            ws_test_app.recv_disconnected_one(client2).await;
        }

        info!("GET UNBANNED");
        ws_test_app.add_time(CON_BLOCK_DURATION).await;

        let client = ws_test_app
            .create_client(0, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;
        ws_test_app.recv_con_allow(client2, client).await;
        ws_test_app.recv_ip_unban(client2, client).await;
        ws_test_app.recv_connected_one(client2, client).await;

        for i in 1..CON_MAX_AMOUNT as usize {
            let client = ws_test_app
                .create_client(i, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
                .await;
            ws_test_app.recv_con_allow(client2, client).await;
            ws_test_app.recv_connected_one(client2, client).await;
        }

        ws_test_app.close().await;
    }

    #[tokio::test]
    async fn throttle_con_flicker_ban() {
        init_tracer();

        let time = Utc::now();
        let mut ws_test_app = WsTestApp::new(4, time).await;

        let client2 = ws_test_app
            .create_client(200, Ipv4Addr::new(0, 0, 0, 2), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.send_live_stats_on(client2).await;
        ws_test_app.recv_connections(client2).await;
        ws_test_app.recv_connected_one(client2, client2).await;

        info!("GET BLOCKED");

        for i in 0..CON_FLICKER_MAX as usize {
            let client = ws_test_app
                .create_client(0, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
                .await;
            ws_test_app.recv_con_allow(client2, client).await;
            ws_test_app.recv_connected_one(client2, client).await;
            ws_test_app.close_client(client).await;
            ws_test_app.recv_disconnected_one(client2).await;
        }

        let client = ws_test_app
            .create_client(0, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_ERR)
            .await;
        ws_test_app.recv_con_banned(client2, client).await;
        ws_test_app
            .recv_ip_banned(client2, client, global::IpBanReason::WsConFlickerDetected)
            .await;

        ws_test_app.close().await;
    }

    #[tokio::test]
    async fn throttle_con_manual_ban() {
        init_tracer();

        let time = Utc::now();
        let mut ws_test_app = WsTestApp::new(5, time).await;

        let client1_ip = Ipv4Addr::new(0, 0, 0, 1);
        let ban_duration = TimeDelta::try_seconds(10).unwrap();
        let ban_date = time + ban_duration;
        let ban_reason = global::IpBanReason::Other(String::from("too stinky"));

        let client1 = ws_test_app
            .create_client(1, client1_ip, CLIENT_CONNECTED_SUCCESS)
            .await;

        let client2 = ws_test_app
            .create_client(200, Ipv4Addr::new(0, 0, 0, 2), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.send_live_stats_on(client2).await;
        ws_test_app.recv_connections(client2).await;
        ws_test_app.recv_connected(client2).await;

        ws_test_app
            .send_custom_msg_once(
                client2,
                global::ClientMsg::BanIp {
                    ip: IpAddr::V4(client1_ip),
                    date: ban_date,
                    reason: ban_reason.clone(),
                },
            )
            .await;

        ws_test_app.recv_req_allow(client2).await;
        ws_test_app
            .recv_ip_banned(client2, client1, ban_reason)
            .await;

        ws_test_app.recv_disconnected_one(client2).await;
        ws_test_app.recv_command_disconnected(client1).await;

        let client1 = ws_test_app
            .create_client(1, client1_ip, CLIENT_CONNECTED_ERR)
            .await;

        ws_test_app.add_time(ban_duration).await;

        let client1 = ws_test_app
            .create_client(1, client1_ip, CLIENT_CONNECTED_SUCCESS)
            .await;
        ws_test_app.recv_con_allow(client2, client1).await;
        ws_test_app.recv_ip_unban(client2, client1).await;
        ws_test_app.recv_connected_one(client2, client1).await;

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;

        //ws_test_app.recv_command_disconnected(client1).await;

        ws_test_app.close().await;
    }

    #[tokio::test]
    async fn http_ban() {
        init_tracer();
        let server = create_server("./artcord-actix/gallery/", "./artcord-actix/assets").await;

        tokio::spawn(server);

        let body = reqwest::get("http://localhost:3000")
            .await
            .unwrap();

        trace!("{:#?}", body);

        let body = reqwest::get("http://localhost:3000")
            .await
            .unwrap();

        trace!("{:#?}", body);

        
    }

    fn init_tracer() {
        let _ = tracing_subscriber::fmt()
            .event_format(
                tracing_subscriber::fmt::format()
                    .with_file(true)
                    .with_line_number(true),
            )
            .with_env_filter(
                env::var("RUST_LOG")
                    .map(|data| tracing_subscriber::EnvFilter::from_str(&data).unwrap())
                    .unwrap_or(tracing_subscriber::EnvFilter::from_str("artcord=trace").unwrap()),
            )
            .try_init();
    }

    async fn on_client_recv(
        write: &mut futures::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
        msg: DebugClientMsg,
    ) -> bool {
        match msg {
            DebugClientMsg::Send((key, msg)) => {
                info!("sending: {msg:#?}");
                let bytes = global::ClientMsg::as_vec(&(key, msg)).unwrap();
                write.send(Message::Binary(bytes)).await.unwrap();
            }
            DebugClientMsg::Disconnect => {
                return true;
            }
        }
        false
    }

    async fn on_read(
        msg: Option<Result<Message, tokio_tungstenite::tungstenite::Error>>,
        server_tx: &mpsc::Sender<(u128, global::ServerMsg)>,
    ) -> bool {
        let Some(msg) = msg else {
            return true;
        };
        let msg = msg.unwrap();

        match msg {
            Message::Binary(msg) => {
                let msg = global::ServerMsg::from_bytes(&msg).unwrap();
                server_tx.send(msg).await.unwrap();
            }
            _ => {}
        }

        false
    }

    async fn drop_db(database_name: impl AsRef<str>, mongo_url: impl AsRef<str>) {
        let mut client_options = ClientOptions::parse(mongo_url).await.unwrap();
        client_options.app_name = Some("My App".to_string());
        let client = mongodb::Client::with_options(client_options).unwrap();

        let db_exists = client
            .list_database_names(doc! {}, None)
            .await
            .unwrap()
            .iter()
            .any(|a| *a == database_name.as_ref());
        let database = client.database(database_name.as_ref());
        if db_exists {
            database.drop(None).await.unwrap();
        }
    }

    #[derive(Error, Debug, PartialEq)]
    pub enum ClientErr {
        #[error("failed to connect")]
        FailedToConnect,

        #[error("exploded")]
        Boom,
    }

    #[derive(Error, Debug)]
    pub enum ClockErr {
        #[error("failed to recv time: {0}")]
        Recv(#[from] oneshot::error::RecvError),

        #[error("failed to request time: {0}")]
        Send(#[from] mpsc::error::SendError<oneshot::Sender<DateTime<Utc>>>),
    }
}
