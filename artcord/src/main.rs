use artcord_actix::server::create_server;
use artcord_mongodb::database::DB;
use artcord_serenity::create_bot::create_bot;
use artcord_state::message::prod_client_msg::ProdThreshold;
use artcord_state::misc::throttle_connection::IpBanReason;
use artcord_state::misc::throttle_threshold::Threshold;
use artcord_state::util::time::Clock;
use artcord_tungstenite::ws::ProdUserAddrMiddleware;
use artcord_tungstenite::ws::Ws;
use artcord_tungstenite::WsThreshold;
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
    //tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::builder().parse_lossy("artcord=trace")).try_init().unwrap();
    // cfg_if! {
    //     if #[cfg(feature = "production")] {
    //         tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init().unwrap();
    //     } else {
    //         tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init().unwrap();
    //     }
    // }

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
    let time_machine = Clock::new();

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
            let threshold = WsThreshold {
                ws_max_con_threshold: Threshold::new_const(10000, TimeDelta::try_minutes(1)),
                ws_max_con_ban_duration: match TimeDelta::try_days(1) {
                    Some(delta) => delta,
                    None => panic!("invalid delta"),
                },
                ws_max_con_threshold_range: 100,
                ws_max_con_ban_reason: IpBanReason::WsTooManyReconnections,
                ws_con_flicker_threshold: Threshold::new_const(10000, TimeDelta::try_minutes(1)),
                ws_con_flicker_ban_duration: match TimeDelta::try_days(1) {
                    Some(delta) => delta,
                    None => panic!("invalid delta"),
                },
                ws_con_flicker_ban_reason: IpBanReason::WsConFlickerDetected,
                ws_req_ban_threshold: Threshold::new_const(10000, TimeDelta::try_minutes(1)),
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
            ProdThreshold,
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

        // let r = try_join!(
        //     async { web_server.await.or_else(|e| Err(e.to_string())) },
        //     async {
        //         web_sockets_handle.await;
        //         Ok(())
        //     },
        //     async {
        //         discord_bot.start().await;
        //         Ok(())
        //     },
        // );
        // r.unwrap();
    } else {
        error!("DISCORD_BOT_TOKEN in .env is missing, bot will not start.");
        select! {
            _ = web_sockets_handle => {},
            _ = web_server => {},
            _ = signal::ctrl_c() => {},
        }
        // let r = try_join!(
        //     async { web_server.await.or_else(|e| Err(e.to_string())) },
        //     async {
        //         web_sockets_handle.await;
        //         Ok(())
        //     },
        // );

        // r.unwrap();
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
        net::{IpAddr, Ipv4Addr},
        str::FromStr,
        sync::Arc,
        time::Duration,
    };

    use artcord_mongodb::database::DB;
    use artcord_state::{
        message::{
            prod_client_msg::{ClientMsg, ClientThresholdMiddleware},
            prod_server_msg::ServerMsg,
        },
        misc::{throttle_connection::IpBanReason, throttle_threshold::Threshold},
        util::time::TimeMiddleware,
    };
    use artcord_tungstenite::ws::{GetUserAddrMiddleware, Ws};
    use artcord_tungstenite::WsThreshold;
    use chrono::{DateTime, TimeDelta, Utc};
    use futures::{stream::SplitSink, SinkExt, StreamExt};
    use mongodb::{bson::doc, options::ClientOptions};
    use std::net::SocketAddr;
    use thiserror::Error;
    use tokio::sync::mpsc;
    use tokio::sync::oneshot;
    use tokio::{net::TcpStream, sync::Mutex};
    use tokio::{select, time::sleep};
    use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
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
        server_rx: mpsc::Receiver<(u128, ServerMsg)>,
        connection_rx: mpsc::Receiver<Result<ConnectionMsg, ClientErr>>,
    }

    #[derive(Debug, PartialEq, Clone)]
    enum DebugClientMsg {
        Send((u128, ClientMsg)),
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

    impl TimeMiddleware for TestClock {
        async fn get_time(&self) -> DateTime<Utc> {
            let (time_tx, time_rx) = oneshot::channel::<DateTime<Utc>>();
            self.time_tx.send(time_tx).await.unwrap();
            time_rx.await.unwrap()
        }
    }

    impl ClientThresholdMiddleware for DebugThreshold {
        fn get_threshold(&self, msg: &ClientMsg) -> Threshold {
            match msg {
                _ => Threshold::new(REQ_MAX_ALLOW, REQ_ALLOW_DURATION),
            }
        }
    }

    // async fn create_connection(
    //     task_tracker: TaskTracker,
    //     cancellation_token: CancellationToken,
    //     result: mpsc::Sender<Result<DebugClientStatusMsg, ClientErr>>,
    // ) -> Connection {

    // }
    // pub fn new() -> (
    //     Self,
    //     mpsc::Sender<(u128, ServerMsg)>,
    //     mpsc::Receiver<DebugClientMsg>,
    // )

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
        pub async fn new(ws_id: usize) -> Self {
            let mongo_name = format!("artcord_test_{}", ws_id);
            drop_db(mongo_name.clone(), MONGO_URL).await;
            let db = DB::new(mongo_name, MONGO_URL).await;
            let db = Arc::new(db);
            let (time_machine, mut time_rx) = TestClock::new();
            let time: Arc<Mutex<DateTime<Utc>>> = Arc::new(Mutex::new(Utc::now()));
            let tracker = TaskTracker::new();
            let cancelation_token = CancellationToken::new();
            let threshold = WsThreshold {
                ws_max_con_threshold: Threshold::new(CON_MAX_BLOCK_AMOUNT, CON_BLOCK_DURATION),
                ws_max_con_ban_duration: CON_BAN_DURATION,
                ws_max_con_threshold_range: CON_MAX_AMOUNT,
                ws_max_con_ban_reason: IpBanReason::WsTooManyReconnections,
                ws_con_flicker_threshold: Threshold::new(
                    CON_FLICKER_MAX,
                    CON_FLICKER_BLOCK_DURATION,
                ),
                ws_con_flicker_ban_duration: CON_FLICKER_BAN_DURATION,
                ws_con_flicker_ban_reason: IpBanReason::WsConFlickerDetected,
                ws_req_ban_threshold: Threshold::new(REQ_MAX_BLOCK, REQ_BLOCK_DURATION),
                ws_req_ban_duration: REQ_BAN_DURATION,
            };

            let (addr_middleware_rx, addr_middleware) = TestUserAddrMiddleware::new();

            tracker.spawn({
                let time = time.clone();
                async move {
                    while let Some(time_rx) = time_rx.recv().await {
                        let time = *time.lock().await;
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

            //let id = self.connections.len();
            self.connections.insert(id, con);
            id
            //let pos = self.connections.iter().position(|c| c.is_none());

            // if let Some((pos, c)) = pos.and_then(|pos| self.connections.get_mut(pos).map(|con| (pos, con))) {
            //     *c = Some(con);
            //     pos
            // } else {
            //     let id = self.connections.len().saturating_sub(1);
            //     self.connections.push(Some(con));
            //     self.ips.insert(id, IpAddr::V4(ip));
            //     id
            // }
            // for c in self.connections.iter_mut() {
            //     if c.is_none() {
            //         *c = Some(con);
            //     }
            // }
            // if !inserted {
            //     self.connections.push(Some(con));
            // }
        }

        // async fn create_client_from_to(&mut self, from: usize, to: usize, ip: Ipv4Addr) {
        //     for id in from..=to {
        //         self.create_client(id, ip).await;
        //     }
        // }

        // async fn recv_client_status(&mut self, client_id: usize) {
        //     let Some(con) = self.connections.get_mut(&client_id) else {
        //         panic!("missing con: {}", client_id);
        //     };
        // }

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

        async fn send(&self, client_id: usize, msg: ClientMsg) {
            let Some(con) = self.connections.get(&client_id) else {
                panic!("missing con: {}", client_id);
            };
            con.client_tx
                .send(DebugClientMsg::Send((0, msg)))
                .await
                .unwrap();
        }

        async fn send_test_msg_once(&mut self, send_client_id: usize) {
            self.send(send_client_id, ClientMsg::Logout).await;
        }

        // async fn send_and_recv_allow_nth(&mut self, send_client_id: usize, check_client_id: usize, times: usize) {
        //     for _ in 0..times {
        //         self.send_test_msg_once(send_client_id).await;
        //         let msg = self.recv(check_client_id).await;
        //         assert!(matches!(
        //             msg,
        //             ServerMsg::WsLiveStatsConReqAllowed {
        //                 con_id,
        //                 path,
        //                 total_amount
        //             }
        //         ));
        //     }
        // }

        // async fn send_and_recv_max_allow(&mut self, send_client_id: usize, check_client_id: usize) {
        //     self.send_and_recv_allow_nth(send_client_id, check_client_id, 5).await;
        // }

        // async fn send_and_recv_allow_once(&mut self, send_client_id: usize, check_client_id: usize) {
        //     self.send_and_recv_allow_nth(send_client_id, check_client_id, 1).await;
        // }

        // async fn send_max_allow_one_off(&mut self, send_client_id: usize, check_client_id: usize) {
        //     self.send_and_recv_allow_nth(send_client_id, check_client_id, 4).await;
        // }

        // async fn send_and_recv_max_block(&mut self, send_client_id: usize, check_client_id: usize) {
        //     for _ in 0..10 {
        //         self.send(send_client_id, ClientMsg::Logout).await;
        //         let msg = self.recv(check_client_id).await;
        //         assert!(matches!(
        //             msg,
        //             ServerMsg::WsLiveStatsConReqBlocked {
        //                 con_id,
        //                 path,
        //                 total_amount
        //             }
        //         ));
        //     }
        // }

        // async fn send_and_recv_ban(&mut self, send_client_id: usize, check_client_id: usize) {
        //     self.send(send_client_id, ClientMsg::Logout).await;
        //     let msg = self.recv(check_client_id).await;
        //     assert!(matches!(
        //         msg,
        //         ServerMsg::WsLiveStatsIpBanned { ip, date, reason }
        //     ));
        //     let msg = self.recv(check_client_id).await;
        //     assert!(matches!(
        //         msg,
        //         ServerMsg::WsLiveStatsConReqBanned { con_id, path, total_amount }
        //     ));
        // }

        async fn set_time_unblocked(&self) {
            let time = &mut (*self.time.lock().await);
            *time += REQ_ALLOW_DURATION;
        }

        async fn add_time(&self, add_this_time: TimeDelta) {
            let time = &mut (*self.time.lock().await);
            *time += add_this_time;
        }

        async fn recv(&mut self, client_id: usize) -> ServerMsg {
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

        async fn recv_command(&mut self, client_id: usize) -> Result<ConnectionMsg, ClientErr> {
            let Some(con) = self.connections.get_mut(&client_id) else {
                panic!("missing con: {}", client_id);
            };

            let msg = con.recv_command().await;
            debug!("recv: {:#?}", msg);
            msg
        }

        async fn send_live_stats_on(&self, client_id: usize) {
            self.send(client_id, ClientMsg::LiveWsStats(true)).await;
        }

        async fn send_live_stats_off(&self, client_id: usize) {
            self.send(client_id, ClientMsg::LiveWsStats(false)).await;
        }

        async fn recv_connections(&mut self, client_id: usize) {
            let msg = self.recv(client_id).await;
            assert!(matches!(msg, ServerMsg::WsLiveStatsIpCons(_)));
        }

        async fn recv_connected(&mut self, client_id: usize) {
            //let mut received: Vec<IpAddr> = Vec::new();
            let ips: Vec<IpAddr> = self.connections.values().map(|con| con.ip).collect();
            trace!("current ips set: {:#?}", ips);
            let good = |msg: ServerMsg| match msg {
                ServerMsg::WsLiveStatsConnected {
                    ip,
                    socket_addr,
                    con_id,
                    banned_until,
                    req_stat,
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
                ServerMsg::WsLiveStatsConnected {
                    ip,
                    socket_addr,
                    con_id,
                    banned_until,
                    req_stat,
                } => ip == targer_ip,
                _ => false,
            })
        }

        async fn recv_disconnected_one(&mut self, client_id: usize) {
            let msg = self.recv(client_id).await;
            assert!(match msg {
                ServerMsg::WsLiveStatsDisconnected { con_id } => true,
                _ => false,
            })
        }

        async fn recv_req_allow(&mut self, client_id: usize) {
            let msg = self.recv(client_id).await;
            assert!(match msg {
                ServerMsg::WsLiveStatsReqAllowed {
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
                ServerMsg::WsLiveStatsReqBlocked {
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
                ServerMsg::WsLiveStatsReqBanned {
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
                ServerMsg::WsLiveStatsConAllowed { ip, total_amount } => ip == c_ip,
                _ => false,
            })
        }

        async fn recv_con_block(&mut self, client_id: usize, target: usize) {
            let c_ip = self.connections.get(&target).map(|con| con.ip).unwrap();
            let msg = self.recv(client_id).await;
            assert!(match msg {
                ServerMsg::WsLiveStatsConBlocked { ip, total_amount } => ip == c_ip,
                _ => false,
            })
        }

        async fn recv_con_banned(&mut self, client_id: usize, target: usize) {
            let c_ip = self.connections.get(&target).map(|con| con.ip).unwrap();
            let msg = self.recv(client_id).await;
            assert!(match msg {
                ServerMsg::WsLiveStatsConBanned { ip, total_amount } => ip == c_ip,
                _ => false,
            })
        }

        async fn recv_ip_banned(&mut self, client_id: usize, target: usize) {
            let c_ip = self.connections.get(&target).map(|con| con.ip).unwrap();
            let msg = self.recv(client_id).await;
            assert!(match msg {
                ServerMsg::WsLiveStatsIpBanned { ip, date, reason } => ip == c_ip,
                _ => false,
            })
        }

        async fn recv_ip_unban(&mut self, client_id: usize, target: usize) {
            let c_ip = self.connections.get(&target).map(|con| con.ip).unwrap();
            let msg = self.recv(client_id).await;
            assert!(match msg {
                ServerMsg::WsLiveStatsIpUnbanned { ip } => ip == c_ip,
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
            let (server_tx, server_rx) = mpsc::channel::<(u128, ServerMsg)>(100);

            // (
            //     Self {
            //         key: 0,
            //         client_tx,
            //         server_rx,
            //     },
            //     server_tx,
            //     client_rx,
            // )

            // let (channel, server_tx, mut client_rx) = Connection::new();
            // // let (client_send_tx, mut client_recv_tx) = mpsc::channel::<(u128, ClientMsg)>(1);
            // // let (server_send_tx, mut server_recv_tx) = mpsc::channel::<(u128, ServerMsg)>(1);

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
            self.connection_rx.recv().await.unwrap()
        }

        pub async fn send(&mut self, msg: ClientMsg) {
            self.client_tx
                .send(DebugClientMsg::Send((self.key, msg)))
                .await
                .unwrap();
        }

        pub async fn recv(&mut self) -> ServerMsg {
            let (_, msg) = self.server_rx.recv().await.unwrap();
            msg
        }
    }

    #[tokio::test]
    async fn throttle_connect_and_disconnect() {
        init_tracer();

        let mut ws_test_app = WsTestApp::new(1).await;

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
    async fn throttle_multi_req() {
        init_tracer();

        let mut ws_test_app = WsTestApp::new(2).await;

        let client1 = ws_test_app
            .create_client(1, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;

        let client2 = ws_test_app
            .create_client(2, Ipv4Addr::new(0, 0, 0, 2), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.send_live_stats_on(client2).await;
        ws_test_app.recv_connections(client2).await;
        ws_test_app.recv_connected(client2).await;

        for _ in 0..REQ_MAX_ALLOW {
            ws_test_app.send_test_msg_once(client1).await;
            ws_test_app.recv_req_allow(client2).await;
        }
        for _ in 0..REQ_MAX_BLOCK {
            ws_test_app.send_test_msg_once(client1).await;
            ws_test_app.recv_req_block(client2).await;
        }

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_ban(client2).await;
        ws_test_app.recv_ip_banned(client2, client1).await;

        let client3 = ws_test_app
            .create_client(3, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.recv_con_allow(client2, client3).await;
        ws_test_app.recv_connected_one(client2, client3).await;
        ws_test_app.send_test_msg_once(client3).await;
        ws_test_app.recv_req_allow(client2).await;

        ws_test_app.add_time(REQ_BAN_DURATION).await;

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;
        // ws_test_app.recv_req_allow(client2).await;
        ws_test_app.recv_ip_unban(client2, client1).await;

        ws_test_app.send_test_msg_once(client3).await;
        ws_test_app.recv_req_allow(client2).await;

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;

        ws_test_app.send_test_msg_once(client3).await;
        ws_test_app.recv_req_allow(client2).await;

        ws_test_app.close().await;
    }

    #[tokio::test]
    async fn throttle_req_ban() {
        init_tracer();

        let mut ws_test_app = WsTestApp::new(2).await;

        let client1 = ws_test_app
            .create_client(1, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;
        let client2 = ws_test_app
            .create_client(2, Ipv4Addr::new(0, 0, 0, 2), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.send_live_stats_on(client2).await;
        ws_test_app.recv_connections(client2).await;
        ws_test_app.recv_connected(client2).await;

        for _ in 0..REQ_MAX_ALLOW {
            ws_test_app.send_test_msg_once(client1).await;
            ws_test_app.recv_req_allow(client2).await;
        }
        for _ in 0..REQ_MAX_BLOCK {
            ws_test_app.send_test_msg_once(client1).await;
            ws_test_app.recv_req_block(client2).await;
        }
        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_ban(client2).await;
        ws_test_app.recv_ip_banned(client2, client1).await;

        ws_test_app.recv_disconnected_one(client2).await;

        info!("point -10");

        let client3 = ws_test_app
            .create_client(3, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_ERR)
            .await;
        info!("point -11");
        ws_test_app.recv_command_disconnected(client1).await;
        info!("point -12");
        ws_test_app.recv_command_disconnected(client3).await;
        
        // ws_test_app.close().await;

        // info!("point -11");

        // return;

        // ws_test_app.recv_con_allow(client2, client3).await;
        // ws_test_app.recv_connected_one(client2, client3).await;

        // info!("point -2");
        // ws_test_app.send_test_msg_once(client3).await;

        //let con_status = ws_test_app.recv_command(client3).await;

        //ws_test_app.recv_req_allow(client2).await;

        info!("point -1");
        sleep(Duration::from_secs(2)).await;

        ws_test_app.add_time(REQ_BAN_DURATION).await;

        let client1 = ws_test_app
            .create_client(1, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;

        info!("point 0");
        ws_test_app.recv_con_allow(client2, client3).await;
        ws_test_app.recv_connected_one(client2, client3).await;

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;
        ws_test_app.recv_ip_unban(client2, client1).await;
        // ws_test_app.recv_req_allow(client2).await;
        // ws_test_app.recv_ip_unban(client2, client1).await;

        info!("point 1");

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;

        // ws_test_app.send_test_msg_once(client3).await;
        // ws_test_app.recv_req_allow(client2).await;

        // for _ in 0..REQ_MAX_ALLOW {
        //     ws_test_app.send_test_msg_once(client3).await;
        //     ws_test_app.recv_req_allow(client2).await;
        // }
        // for _ in 0..REQ_MAX_BLOCK {
        //     ws_test_app.send_test_msg_once(client3).await;
        //     ws_test_app.recv_req_block(client2).await;
        // }

        // ws_test_app.send_live_stats_on(client2).await;
        // ws_test_app.recv_connections(client2).await;
        // ws_test_app.recv_connected(client2).await;

        // ws_test_app.close_client(client1).await;
        // ws_test_app.recv_disconnected_one(client2).await;

        // let client1 = ws_test_app
        //     .create_client(1, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
        //     .await;

        // ws_test_app.recv_con_allow(client2, client1).await;
        // ws_test_app.recv_connected_one(client2, client1).await;

        ws_test_app.close().await;
    }

    #[tokio::test]
    async fn ws_throttle_req_test() {
        init_tracer();

        let mut ws_test_app = WsTestApp::new(69).await;

        let client1 = ws_test_app
            .create_client(1, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;
        let client2 = ws_test_app
            .create_client(2, Ipv4Addr::new(0, 0, 0, 2), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.send_live_stats_on(client2).await;
        ws_test_app.recv_connections(client2).await;
        ws_test_app.recv_connected(client2).await;

        let client3 = ws_test_app
            .create_client(3, Ipv4Addr::new(0, 0, 0, 3), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.recv_con_allow(client2, client3).await;
        ws_test_app.recv_connected_one(client2, client3).await;

        for _ in 0..REQ_MAX_ALLOW {
            ws_test_app.send_test_msg_once(client1).await;
            ws_test_app.recv_req_allow(client2).await;
        }
        for _ in 0..REQ_MAX_BLOCK {
            ws_test_app.send_test_msg_once(client1).await;
            ws_test_app.recv_req_block(client2).await;
        }
        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_ban(client2).await;
        ws_test_app.recv_ip_banned(client2, client1).await;

        ws_test_app.add_time(REQ_BAN_DURATION).await;

        ws_test_app.send_test_msg_once(client1).await;
        ws_test_app.recv_req_allow(client2).await;
        ws_test_app.recv_ip_unban(client2, client1).await;
        //ws_test_app.send_max_allow_one_off(client1, client2).await;

        ws_test_app.close_client(client1).await;
        ws_test_app.recv_disconnected_one(client2).await;

        let client1 = ws_test_app
            .create_client(client1, Ipv4Addr::new(0, 0, 0, 1), CLIENT_CONNECTED_SUCCESS)
            .await;

        ws_test_app.recv_con_allow(client2, client1).await;
        ws_test_app.recv_connected_one(client2, client1).await;

        for id in 10..CON_MAX_AMOUNT as usize + 10 {
            ws_test_app
                .create_client(id, Ipv4Addr::new(0, 0, 0, 10), CLIENT_CONNECTED_SUCCESS)
                .await;
            ws_test_app.recv_con_allow(client2, id).await;
            ws_test_app.recv_connected_one(client2, id).await;
        }

        for _ in 0..CON_MAX_BLOCK_AMOUNT {
            ws_test_app
                .create_client(15, Ipv4Addr::new(0, 0, 0, 10), CLIENT_CONNECTED_ERR)
                .await;
            ws_test_app.recv_con_block(client2, 15).await;
        }

        ws_test_app
            .create_client(15, Ipv4Addr::new(0, 0, 0, 10), CLIENT_CONNECTED_ERR)
            .await;
        ws_test_app.recv_con_banned(client2, 15).await;
        ws_test_app.recv_ip_banned(client2, 15).await;

        ws_test_app.add_time(CON_BAN_DURATION).await;

        // let client15 = ws_test_app
        //     .create_client(15, Ipv4Addr::new(0, 0, 0, 10), CLIENT_CONNECTED_ERR)
        //     .await;
        // ws_test_app.recv_con_block(client2, client15).await;
        //ws_test_app.recv_ip_unban(client2, 15).await;

        ws_test_app.close_client(14).await;
        ws_test_app.recv_disconnected_one(client2).await;

        let client15 = ws_test_app
            .create_client(15, Ipv4Addr::new(0, 0, 0, 10), CLIENT_CONNECTED_SUCCESS)
            .await;
        ws_test_app.recv_con_allow(client2, client15).await;
        ws_test_app.recv_ip_unban(client2, client15).await;
        ws_test_app.recv_connected_one(client2, client15).await;

        info!("WOOOOOOOOOOOOOOOOW");

        ws_test_app.close().await;

        // let (clinet_1_result_tx, mut client_1_result_rx) = mpsc::channel(100);
        // let mut client = create_connection(
        //     root_task_tracker.clone(),
        //     cancelation_token.clone(),
        //     clinet_1_result_tx,
        // )
        // .await;
        // let ((mut addr, return_tx)) = addr_rx.recv().await.unwrap();
        // let client_1_ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 69));
        // addr.set_ip(client_1_ip);
        // return_tx.send(addr).unwrap();
        // assert_eq!(
        //     client_1_result_rx.recv().await.unwrap(),
        //     Ok(ConnectionMsg::Connected)
        // );

        // let (client_2_result_tx, mut client_2_result_rx) = mpsc::channel(100);
        // let mut client2 = create_connection(
        //     root_task_tracker.clone(),
        //     cancelation_token.clone(),
        //     client_2_result_tx,
        // )
        // .await;
        // let ((mut addr, return_tx)) = addr_rx.recv().await.unwrap();
        // let client_2_ip = IpAddr::V4(Ipv4Addr::new(1, 4, 2, 0));
        // addr.set_ip(client_2_ip);
        // return_tx.send(addr).unwrap();
        // assert_eq!(
        //     client_2_result_rx.recv().await.unwrap(),
        //     Ok(ConnectionMsg::Connected)
        // );

        // let client_3_ip = IpAddr::V4(Ipv4Addr::new(1, 4, 2, 3));

        // let mut posibilities = vec![client_1_ip, client_2_ip, client_3_ip];
        // let mut check_ips = |msg: ServerMsg| match msg {
        //     ServerMsg::WsLiveStatsConnected(stat) => {
        //         let Some(position) = posibilities.iter().position(|ip| stat.ip == *ip) else {
        //             return false;
        //         };
        //         posibilities.remove(position);
        //         true
        //     }
        //     _ => false,
        // };

        // client2.send(ClientMsg::LiveWsStats(true)).await;
        // // client2.send(ClientMsg::LiveWsStats(true)).await;
        // let msg = client2.recv().await;
        // debug!("r: {:#?}", msg);
        // assert!(matches!(msg, ServerMsg::WsLiveStatsIpConnections(_)));

        // let (client_3_result_tx, mut client_3_result_rx) = mpsc::channel(100);
        // let mut client3 = create_connection(
        //     root_task_tracker.clone(),
        //     cancelation_token.clone(),
        //     client_3_result_tx,
        // )
        // .await;
        // let ((mut addr, return_tx)) = addr_rx.recv().await.unwrap();

        // addr.set_ip(client_3_ip);
        // return_tx.send(addr).unwrap();
        // assert_eq!(
        //     client_3_result_rx.recv().await.unwrap(),
        //     Ok(ConnectionMsg::Connected)
        // );

        // // let msg = client2.recv().await;
        // // debug!("r: {:#?}", msg);
        // // assert!(matches!(msg, ServerMsg::WsLiveStatsConnected(_)));

        // let msg = client2.recv().await;
        // debug!("r: {:#?}", msg);
        // assert!(check_ips(msg));

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(check_ips(msg));

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(matches!(
        //     msg,
        //     ServerMsg::WsLiveStatsIpConnectionAllowed { ip, total_amount }
        // ));

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(check_ips(msg));

        // for _ in 0..5 {
        //     client.send(ClientMsg::Logout).await;
        //     let msg = client2.recv().await;
        //     debug!("r2: {:#?}", msg);
        //     assert!(matches!(
        //         msg,
        //         ServerMsg::WsLiveStatsConReqAllowed {
        //             con_id,
        //             path,
        //             total_amount
        //         }
        //     ));
        // }

        // client.send(ClientMsg::Logout).await;
        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(matches!(
        //     msg,
        //     ServerMsg::WsLiveStatsConReqBlocked {
        //         con_id,
        //         path,
        //         total_amount
        //     }
        // ));

        // (*fake_date.lock().await) = Some(now + MSG_THRESHOLD_DELTA);

        // for _ in 0..5 {
        //     client.send(ClientMsg::Logout).await;
        //     let msg = client2.recv().await;
        //     debug!("r2: {:#?}", msg);
        //     assert!(matches!(
        //         msg,
        //         ServerMsg::WsLiveStatsConReqAllowed {
        //             con_id,
        //             path,
        //             total_amount
        //         }
        //     ));
        // }

        // for _ in 0..9 {
        //     client.send(ClientMsg::Logout).await;
        //     let msg = client2.recv().await;
        //     debug!("r2: {:#?}", msg);
        //     assert!(matches!(
        //         msg,
        //         ServerMsg::WsLiveStatsConReqBlocked {
        //             con_id,
        //             path,
        //             total_amount
        //         }
        //     ));
        // }

        // client.send(ClientMsg::Logout).await;

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(matches!(
        //     msg,
        //     ServerMsg::WsLiveStatsIpBanned { ip, date, reason }
        // ));

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(matches!(
        //     msg,
        //     ServerMsg::WsLiveStatsConReqBanned {
        //         con_id,
        //         path,
        //         total_amount
        //     }
        // ));

        // {
        //     let time = &mut (*fake_date.lock().await);
        //     *time = time.map(|time| time + REQ_BAN_DURATION);
        // }

        // client.send(ClientMsg::Logout).await;

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(matches!(msg, ServerMsg::WsLiveStatsIpUnbanned { ip }));

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(matches!(
        //     msg,
        //     ServerMsg::WsLiveStatsConReqAllowed {
        //         con_id,
        //         path,
        //         total_amount
        //     }
        // ));

        // client3.command(DebugClientMsg::Disconnect).await;
        // assert_eq!(
        //     client_3_result_rx.recv().await.unwrap(),
        //     Ok(ConnectionMsg::Disconnected)
        // );

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(matches!(msg, ServerMsg::WsLiveStatsDisconnected { con_id }));

        // client2.send(ClientMsg::LiveWsStats(false)).await;

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(matches!(
        //     msg,
        //     ServerMsg::WsLiveStatsConReqAllowed {
        //         con_id,
        //         path,
        //         total_amount
        //     }
        // ));

        // client.send(ClientMsg::Logout).await;
        // client2.send(ClientMsg::LiveWsStats(true)).await;
        // let msg = client2.recv().await;
        // debug!("r: {:#?}", msg);
        // assert!(matches!(msg, ServerMsg::WsLiveStatsIpConnections(_)));

        // let mut posibilities = vec![client_1_ip, client_2_ip];
        // let mut check_ips = |msg: ServerMsg| match msg {
        //     ServerMsg::WsLiveStatsConnected(stat) => {
        //         let Some(position) = posibilities.iter().position(|ip| stat.ip == *ip) else {
        //             return false;
        //         };
        //         posibilities.remove(position);
        //         true
        //     }
        //     _ => false,
        // };

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(check_ips(msg));

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(check_ips(msg));

        // //client.send(ClientMsg::Logout).await;

        // // let msg = client2.recv().await;
        // // debug!("r2: {:#?}", msg);
        // // assert!( matches!(msg, ServerMsg::WsLiveStatsConReqAllowed { con_id, path, total_amount } ) );

        // // let msg = client2.recv().await;
        // // debug!("r2: {:#?}", msg);
        // // assert!( matches!(msg, ServerMsg::WsLiveThrottleCachedIncPath { ip, path }));

        // // let msg = client2.recv().await;
        // // debug!("r2: {:#?}", msg);
        // // assert!(matches!(msg, ServerMsg::WsLiveStatsStarted(_)));

        // info!("exiting...");
        // cancelation_token.cancel();
        // //web_sockets_handle.await.unwrap();
        // root_task_tracker.close();

        // assert_eq!(
        //     client_1_result_rx.recv().await.unwrap(),
        //     Ok(ConnectionMsg::Disconnected)
        // );
        // //assert_eq!(client_2_result_rx.recv().await.unwrap(), Ok(DebugClientStatusMsg::Disconnected));

        // root_task_tracker.wait().await;
    }

    // #[tokio::test]
    // async fn ws_test() {
    //     tracing_subscriber::fmt()
    //         .event_format(
    //             tracing_subscriber::fmt::format()
    //                 .with_file(true)
    //                 .with_line_number(true),
    //         )
    //         .with_env_filter(tracing_subscriber::EnvFilter::from_str("artcord=trace").unwrap())
    //         .try_init()
    //         .unwrap();

    //     drop_db(MONGO_NAME, MONGO_URL).await;
    //     let db = DB::new(MONGO_NAME, MONGO_URL).await;
    //     let db = Arc::new(db);
    //     let (time_machine, mut time_rx) = TestClock::new();
    //     let fake_date: Arc<Mutex<Option<DateTime<Utc>>>> = Arc::new(Mutex::new(None));
    //     let root_task_tracker = TaskTracker::new();
    //     let cancelation_token = CancellationToken::new();
    //     let now = Utc::now();

    //     let threshold = WsThreshold {
    //         ws_max_con_threshold: Threshold::new_const(10, TimeDelta::try_minutes(1)),
    //         ws_max_con_ban_duration: match TimeDelta::try_minutes(1) {
    //             Some(delta) => delta,
    //             None => panic!("invalid delta"),
    //         },
    //         ws_max_con_threshold_range: 5,
    //         ws_max_con_ban_reason: IpBanReason::WsTooManyReconnections,
    //         ws_con_flicker_threshold: Threshold::new_const(10, TimeDelta::try_minutes(1)),
    //         ws_con_flicker_ban_duration: match TimeDelta::try_minutes(1) {
    //             Some(delta) => delta,
    //             None => panic!("invalid delta"),
    //         },
    //         ws_con_flicker_ban_reason: IpBanReason::WsConFlickerDetected,
    //         ws_req_ban_threshold: Threshold::new_const(10, TimeDelta::try_minutes(1)),
    //         ws_req_ban_duration: match TimeDelta::try_minutes(1) {
    //             Some(delta) => delta,
    //             None => panic!("invalid delta"),
    //         },
    //     };

    //     root_task_tracker.spawn({
    //         let fake_date = fake_date.clone();
    //         async move {
    //             while let Some(time_rx) = time_rx.recv().await {
    //                 let Some(fake_date) = *fake_date.lock().await else {
    //                     time_rx.send(now).unwrap();
    //                     continue;
    //                 };
    //                 time_rx.send(fake_date).unwrap();
    //             }
    //         }
    //     });

    //     let (mut addr_rx, addr_middleware) = TestUserAddrMiddleware::new();

    //     let web_sockets_handle = root_task_tracker.spawn(
    //         Ws::create(
    //             root_task_tracker.clone(),
    //             cancelation_token.clone(),
    //             "0.0.0.0:3420".to_string(),
    //             threshold,
    //             db.clone(),
    //             time_machine,
    //             DebugThreshold,
    //             addr_middleware,
    //         )
    //     );

    //     debug!("ONE 1");

    //     let (clinet_1_result_tx, mut client_1_result_rx) = mpsc::channel(100);
    //     debug!("ONE 2");
    //     let mut client = create_client(root_task_tracker.clone(), cancelation_token.clone(), clinet_1_result_tx).await;
    //     debug!("ONE 3");

    //     debug!("ONE 4");
    //     let ((mut addr, return_tx)) = addr_rx.recv().await.unwrap();
    //     debug!("ONE 5");
    //     let client_1_ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 69));
    //     debug!("ONE 6");
    //     addr.set_ip(client_1_ip);
    //     debug!("ONE 7");
    //     return_tx.send(addr).unwrap();
    //     assert_eq!(client_1_result_rx.recv().await.unwrap(), Ok(DebugClientMsg::Connected));

    //     debug!("TWO");

    //     let (client_2_result_tx, mut client_2_result_rx) = mpsc::channel(100);
    //     let mut client2 = create_client(root_task_tracker.clone(), cancelation_token.clone(), client_2_result_tx).await;

    //     let ((mut addr, return_tx)) = addr_rx.recv().await.unwrap();
    //     let client_2_ip = IpAddr::V4(Ipv4Addr::new(1, 4, 2, 0));
    //     addr.set_ip(client_2_ip);
    //     return_tx.send(addr).unwrap();
    //     assert_eq!(client_2_result_rx.recv().await.unwrap(), Ok(DebugClientMsg::Connected));

    //     debug!("THREE");

    //     client2.send(ClientMsg::LiveWsThrottleCache(true)).await;
    //     client2.send(ClientMsg::LiveWsStats(true)).await;
    //     let msg = client2.recv().await;
    //     debug!("r: {:#?}", msg);
    //     let r = match msg {
    //         ServerMsg::WsLiveThrottleCachedEntryAdded(stats) => {
    //             stats.contains_key(&client_1_ip) && stats.contains_key(&client_2_ip)
    //         }
    //         _ => false
    //     };
    //     assert!(r);
    //     let msg = client2.recv().await;
    //     debug!("r2: {:#?}", msg);
    //     assert!( matches!(msg, ServerMsg::WsLiveThrottleCachedIncPath { ip, path }));

    //     let msg = client2.recv().await;
    //     debug!("r2: {:#?}", msg);
    //     assert!(matches!(msg, ServerMsg::WsLiveStatsStarted(_)));

    //     for _ in 0..4 {
    //         client
    //             .send(ClientMsg::WsStatsWithPagination {
    //                 page: 0,
    //                 amount: 10,
    //             })
    //             .await;
    //         let msg = client.recv().await;

    //         assert_eq!(
    //             msg,
    //             ServerMsg::WsStatsWithPagination {
    //                 total_count: 0,
    //                 latest: None,
    //                 stats: Vec::new()
    //             }
    //         );

    //         let msg = client2.recv().await;
    //         debug!("r2: {:#?}", msg);
    //         assert!( matches!(msg, ServerMsg::WsLiveThrottleCachedIncPath { ip, path }));

    //         let msg = client2.recv().await;
    //         debug!("r2: {:#?}", msg);
    //         assert!(matches!(msg, ServerMsg::WsLiveStatsUpdateInc { con_key, path }));
    //         //debug!("recv: {:#?}", msg);
    //     }

    //     client
    //         .send(ClientMsg::WsStatsWithPagination {
    //             page: 0,
    //             amount: 10,
    //         })
    //         .await;
    //     let msg = client.recv().await;

    //     assert_eq!(
    //         msg,
    //         ServerMsg::WsStatsWithPagination {
    //             total_count: 0,
    //             latest: None,
    //             stats: Vec::new()
    //         }
    //     );

    //     let msg = client2.recv().await;
    //     debug!("r2: {:#?}", msg);
    //     assert!( matches!(msg, ServerMsg::WsLiveThrottleCachedIncPath { ip, path }));

    //     let msg = client2.recv().await;
    //     debug!("r2: {:#?}", msg);
    //     assert!(matches!(msg, ServerMsg::WsLiveStatsUpdateInc { con_key, path }));

    //     client
    //         .send(ClientMsg::WsStatsWithPagination {
    //             page: 0,
    //             amount: 10,
    //         })
    //         .await;
    //     let msg = client.recv().await;

    //     assert_eq!(msg, ServerMsg::TooManyRequests);

    //     let msg = client2.recv().await;
    //     debug!("r2: {:#?}", msg);
    //     assert!( matches!(msg, ServerMsg::WsLiveThrottleCachedIncPath { ip, path }));

    //     let msg = client2.recv().await;
    //     debug!("r2: {:#?}", msg);
    //     assert!(matches!(msg, ServerMsg::WsLiveStatsUpdateInc { con_key, path }));

    //     //assert!( matches!(msg, ServerMsg::WsLiveThrottle));

    //     (*fake_date.lock().await) = Some(now + MSG_THRESHOLD_DELTA);

    //     client
    //         .send(ClientMsg::WsStatsWithPagination {
    //             page: 0,
    //             amount: 10,
    //         })
    //         .await;
    //     let msg = client.recv().await;

    //     assert_eq!(
    //         msg,
    //         ServerMsg::WsStatsWithPagination {
    //             total_count: 0,
    //             latest: None,
    //             stats: Vec::new()
    //         }
    //     );

    //     let msg = client2.recv().await;
    //     debug!("r2: {:#?}", msg);
    //     assert!( matches!(msg, ServerMsg::WsLiveThrottleCachedIncPath { ip, path }));

    //     let msg = client2.recv().await;
    //     debug!("r2: {:#?}", msg);
    //     assert!(matches!(msg, ServerMsg::WsLiveStatsUpdateInc { con_key, path }));

    //     for _ in 0..4 {
    //         client
    //             .send(ClientMsg::WsStatsWithPagination {
    //                 page: 0,
    //                 amount: 10,
    //             })
    //             .await;
    //         let msg = client.recv().await;

    //         assert_eq!(
    //             msg,
    //             ServerMsg::WsStatsWithPagination {
    //                 total_count: 0,
    //                 latest: None,
    //                 stats: Vec::new()
    //             }
    //         );

    //         let msg = client2.recv().await;
    //         debug!("r2: {:#?}", msg);
    //         assert!( matches!(msg, ServerMsg::WsLiveThrottleCachedIncPath { ip, path }));

    //         let msg = client2.recv().await;
    //         debug!("r2: {:#?}", msg);
    //         assert!(matches!(msg, ServerMsg::WsLiveStatsUpdateInc { con_key, path }));
    //         //debug!("recv: {:#?}", msg);
    //     }

    //     for _ in 0..10 {
    //         client
    //             .send(ClientMsg::WsStatsWithPagination {
    //                 page: 0,
    //                 amount: 10,
    //             })
    //             .await;
    //         let msg = client.recv().await;

    //         assert_eq!(
    //             msg,
    //             ServerMsg::TooManyRequests
    //         );

    //         let msg = client2.recv().await;
    //         debug!("r2: {:#?}", msg);
    //         assert!( matches!(msg, ServerMsg::WsLiveThrottleCachedIncPath { ip, path }));

    //         let msg = client2.recv().await;
    //         debug!("r2: {:#?}", msg);
    //         assert!(matches!(msg, ServerMsg::WsLiveStatsUpdateInc { con_key, path }));
    //         //debug!("recv: {:#?}", msg);
    //     }

    //     let msg = client2.recv().await;
    //     debug!("r2: {:#?}", msg);
    //     assert!(matches!(msg, ServerMsg::WsLiveThrottleCachedBanned { ip, date, reason }));

    //     info!("exiting...");
    //     cancelation_token.cancel();
    //     //web_sockets_handle.await.unwrap();
    //     root_task_tracker.close();
    //     root_task_tracker.wait().await;
    // }

    fn init_tracer() {
        let _ = tracing_subscriber::fmt()
            .event_format(
                tracing_subscriber::fmt::format()
                    .with_file(true)
                    .with_line_number(true),
            )
            .with_env_filter(tracing_subscriber::EnvFilter::from_str("artcord=trace").unwrap())
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
                let bytes = ClientMsg::as_vec(&(key, msg)).unwrap();
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
        server_tx: &mpsc::Sender<(u128, ServerMsg)>,
    ) -> bool {
        let Some(msg) = msg else {
            return true;
        };
        let msg = msg.unwrap();

        match msg {
            Message::Binary(msg) => {
                let msg = ServerMsg::from_bytes(&msg).unwrap();
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
    }

    #[derive(Error, Debug)]
    pub enum ClockErr {
        #[error("failed to recv time: {0}")]
        Recv(#[from] oneshot::error::RecvError),

        #[error("failed to request time: {0}")]
        Send(#[from] mpsc::error::SendError<oneshot::Sender<DateTime<Utc>>>),
    }
}
