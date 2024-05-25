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
use tracing::Instrument;
use std::{env, sync::Arc};
use tokio::select;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::error;
use tracing::info;
use tracing::trace;

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
        ).instrument(tracing::trace_span!(
            "ws",
            "{}",
            ws_ip,
        ))
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
    use std::{net::{IpAddr, Ipv4Addr}, str::FromStr, sync::Arc};

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
    use artcord_tungstenite::{ WsThreshold};
    use chrono::{DateTime, TimeDelta, Utc};
    use futures::{stream::SplitSink, SinkExt, StreamExt};
    use mongodb::{bson::doc, options::ClientOptions};
    use std::net::SocketAddr;
    use thiserror::Error;
    use tokio::select;
    use tokio::sync::mpsc;
    use tokio::sync::oneshot;
    use tokio::{net::TcpStream, sync::Mutex};
    use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
    use tokio_util::{sync::CancellationToken, task::TaskTracker};
    use tracing::{debug, error, info, Level};

    const MONGO_NAME: &'static str = "artcord_test";
    const MONGO_NAME2: &'static str = "artcord_test2";
    const MONGO_URL: &'static str = "mongodb://root:U2L63zXot4n5@localhost:27017";
    const MSG_THRESHOLD_AMOUNT: u64 = 5;
    const MSG_THRESHOLD_DELTA: TimeDelta = match TimeDelta::try_seconds(10) {
        Some(delta) => delta,
        None => panic!("failed to create delta"),
    };

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub struct DebugThreshold;

    struct Client {
        key: u128,
        client_tx: mpsc::Sender<DebugClientMsg>,
        //client_recv_tx: mpsc::Sender<(u128, ClientMsg)>,
        //server_send_tx: mpsc::Sender<(u128, ServerMsg)>,
        server_rx: mpsc::Receiver<(u128, ServerMsg)>,
    }

    #[derive(Debug, PartialEq, Clone)]
    enum DebugClientMsg {
        Send((u128, ClientMsg)),
        Disconnect,
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    enum DebugClientStatusMsg {
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
                _ => Threshold::new(MSG_THRESHOLD_AMOUNT, MSG_THRESHOLD_DELTA),
            }
        }
    }

    impl Client {
        pub fn new() -> (
            Self,
            mpsc::Sender<(u128, ServerMsg)>,
            mpsc::Receiver<DebugClientMsg>,
        ) {
            let (client_tx, client_rx) = mpsc::channel::<DebugClientMsg>(100);
            let (server_tx, server_rx) = mpsc::channel::<(u128, ServerMsg)>(100);

            (
                Self {
                    key: 0,
                    client_tx,
                    server_rx,
                },
                server_tx,
                client_rx,
            )
        }

        pub async fn command(&mut self, msg: DebugClientMsg) {
            self.client_tx.send(msg).await.unwrap();
        }


        pub async fn send(&mut self, msg: ClientMsg) {
            self.client_tx.send(DebugClientMsg::Send((self.key, msg))).await.unwrap();
        }

        pub async fn recv(&mut self) -> ServerMsg {
            let (_, msg) = self.server_rx.recv().await.unwrap();
            msg
        }
    }


    #[tokio::test]
    async fn ws_throttle_req_test() {
        tracing_subscriber::fmt()
            .event_format(
                tracing_subscriber::fmt::format()
                    .with_file(true)
                    .with_line_number(true),
            )
            .with_env_filter(tracing_subscriber::EnvFilter::from_str("artcord=trace").unwrap())
            .try_init()
            .unwrap();

        drop_db(MONGO_NAME2, MONGO_URL).await;
        let db = DB::new(MONGO_NAME2, MONGO_URL).await;
        let db = Arc::new(db);
        let (time_machine, mut time_rx) = TestClock::new();
        let fake_date: Arc<Mutex<Option<DateTime<Utc>>>> = Arc::new(Mutex::new(None));
        let root_task_tracker = TaskTracker::new();
        let cancelation_token = CancellationToken::new();
        let now = Utc::now();

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

        root_task_tracker.spawn({
            let fake_date = fake_date.clone();
            async move {
                while let Some(time_rx) = time_rx.recv().await {
                    let Some(fake_date) = *fake_date.lock().await else {
                        time_rx.send(now).unwrap();
                        continue;
                    };
                    time_rx.send(fake_date).unwrap();
                }
            }
        });

        let (mut addr_rx, addr_middleware) = TestUserAddrMiddleware::new();

        let web_sockets_handle = root_task_tracker.spawn(
            Ws::create(
                root_task_tracker.clone(),
                cancelation_token.clone(),
                "0.0.0.0:3420".to_string(),
                threshold,
                db.clone(),
                time_machine,
                DebugThreshold,
                addr_middleware,
            )
        );

        let (clinet_1_result_tx, mut client_1_result_rx) = mpsc::channel(100);
        let mut client = create_client(root_task_tracker.clone(), cancelation_token.clone(), clinet_1_result_tx).await;
        let ((mut addr, return_tx)) = addr_rx.recv().await.unwrap();
        let client_1_ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 69));
        addr.set_ip(client_1_ip);
        return_tx.send(addr).unwrap();
        assert_eq!(client_1_result_rx.recv().await.unwrap(), Ok(DebugClientStatusMsg::Connected));

        let (client_2_result_tx, mut client_2_result_rx) = mpsc::channel(100);
        let mut client2 = create_client(root_task_tracker.clone(), cancelation_token.clone(), client_2_result_tx).await;
        let ((mut addr, return_tx)) = addr_rx.recv().await.unwrap();
        let client_2_ip = IpAddr::V4(Ipv4Addr::new(1, 4, 2, 0));
        addr.set_ip(client_2_ip);
        return_tx.send(addr).unwrap();
        assert_eq!(client_2_result_rx.recv().await.unwrap(), Ok(DebugClientStatusMsg::Connected));

        let (client_3_result_tx, mut client_3_result_rx) = mpsc::channel(100);
        let mut client3 = create_client(root_task_tracker.clone(), cancelation_token.clone(), client_3_result_tx).await;
        let ((mut addr, return_tx)) = addr_rx.recv().await.unwrap();
        let client_3_ip = IpAddr::V4(Ipv4Addr::new(1, 4, 2, 3));
        addr.set_ip(client_3_ip);
        return_tx.send(addr).unwrap();
        assert_eq!(client_3_result_rx.recv().await.unwrap(), Ok(DebugClientStatusMsg::Connected));


        client2.send(ClientMsg::LiveWsStats(true)).await;
        // client2.send(ClientMsg::LiveWsStats(true)).await;
        let msg = client2.recv().await;
        debug!("r: {:#?}", msg);
        let mut posibilities = vec![client_1_ip, client_2_ip, client_3_ip];
        let mut check_ips = |msg: ServerMsg| {
            match msg {
                ServerMsg::WsLiveStatsConnected(stat) => {
                    let Some(position) = posibilities.iter().position(|ip| stat.ip == *ip) else {
                         return false;
                    };
                    posibilities.remove(position);
                    true
                }
                _ => false
            }
        };
        assert!(check_ips(msg));

        let msg = client2.recv().await;
        debug!("r2: {:#?}", msg);
        assert!(check_ips(msg));
       

        let msg = client2.recv().await;
        debug!("r2: {:#?}", msg);
        client.send(ClientMsg::Logout).await;

        let msg = client2.recv().await;
        debug!("r2: {:#?}", msg);
        assert!( matches!(msg, ServerMsg::WsLiveStatsConReqAllowed { con_id, path, total_amount } ) );

        client3.command(DebugClientMsg::Disconnect).await;
        assert_eq!(client_3_result_rx.recv().await.unwrap(), Ok(DebugClientStatusMsg::Disconnected));

        let msg = client2.recv().await;
        debug!("r2: {:#?}", msg);
        assert!( matches!(msg, ServerMsg::WsLiveStatsDisconnected { con_id } ) );
        
        client2.send(ClientMsg::LiveWsStats(false)).await;

        let msg = client2.recv().await;
        debug!("r2: {:#?}", msg);
        assert!( matches!(msg, ServerMsg::WsLiveStatsConReqAllowed { con_id, path, total_amount } ) );

        client.send(ClientMsg::Logout).await;

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!( matches!(msg, ServerMsg::WsLiveStatsConReqAllowed { con_id, path, total_amount } ) );

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!( matches!(msg, ServerMsg::WsLiveThrottleCachedIncPath { ip, path }));

        // let msg = client2.recv().await;
        // debug!("r2: {:#?}", msg);
        // assert!(matches!(msg, ServerMsg::WsLiveStatsStarted(_)));
        
        
        info!("exiting...");
        cancelation_token.cancel();
        //web_sockets_handle.await.unwrap();
        root_task_tracker.close();

        assert_eq!(client_1_result_rx.recv().await.unwrap(), Ok(DebugClientStatusMsg::Disconnected));
        //assert_eq!(client_2_result_rx.recv().await.unwrap(), Ok(DebugClientStatusMsg::Disconnected));

        root_task_tracker.wait().await;


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

    async fn create_client(task_tracker: TaskTracker, cancellation_token: CancellationToken, result: mpsc::Sender<Result<DebugClientStatusMsg, ClientErr>>) -> Client {
        let (channel, server_tx, mut client_rx) = Client::new();
        // let (client_send_tx, mut client_recv_tx) = mpsc::channel::<(u128, ClientMsg)>(1);
        // let (server_send_tx, mut server_recv_tx) = mpsc::channel::<(u128, ServerMsg)>(1);

        task_tracker.spawn(async move {
            //debug!("client 1");
            let url = url::Url::parse("ws://localhost:3420").unwrap();
            //debug!("client 2");
            let con = connect_async(url).await;
            //debug!("client 3");
            let Ok((ws_stream, res)) = con else {
                let _ = result.send(Err(ClientErr::FailedToConnect)).await;
                return;
            };
            //debug!("client 4");
            let _ = result.send(Ok(DebugClientStatusMsg::Connected)).await;
            //debug!("client 5");
            let (mut write, mut read) = ws_stream.split();

            loop {
                select! {
                    msg = client_rx.recv() => {
                        let exit = on_client_recv(&mut write, msg.unwrap()).await;
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
            let _ = result.send(Ok(DebugClientStatusMsg::Disconnected)).await;
            debug!("client exited.");
        });

        channel
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
            DebugClientMsg::Send((key,msg)) => {
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
