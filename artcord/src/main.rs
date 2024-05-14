use artcord_actix::server::create_server;
use artcord_mongodb::database::DB;
use artcord_serenity::create_bot::create_bot;
use artcord_state::misc::throttle_threshold::Threshold;
use artcord_tungstenite::ws_app::create_ws;
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

    let task_tracker = TaskTracker::new();
    let cancelation_token = CancellationToken::new();

    let web_server = create_server(&gallery_root_dir, &assets_root_dir).await;

    let threshold = WsThreshold {
        ws_app_threshold: Threshold::new_const(10000, TimeDelta::try_minutes(1)),
        ws_app_ban_duration: match TimeDelta::try_days(1) {
            Some(delta) => delta,
            None => panic!("invalid delta"),
        },
        ws_app_threshold_range: 5,

        ws_stat_threshold: Threshold::new_const(10000, TimeDelta::try_minutes(1)),
        ws_stat_ban_duration: match TimeDelta::try_days(1) {
            Some(delta) => delta,
            None => panic!("invalid delta"),
        },
    };

    let (web_sockets_handle, web_sockets_channel) = create_ws(
        task_tracker.clone(),
        cancelation_token.clone(),
        "0.0.0.0:3420",
        &threshold,
        db.clone(),
    )
    .await;
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
    use std::{str::FromStr, sync::Arc};

    use chrono::TimeDelta;
    use tokio::net::TcpStream;

    use artcord_mongodb::database::DB;
    use artcord_state::{
        message::{prod_client_msg::ClientMsg, prod_server_msg::ServerMsg},
        misc::throttle_threshold::Threshold,
    };
    use artcord_tungstenite::{ws_app::create_ws, WsThreshold};
    use futures::{stream::SplitSink, SinkExt, StreamExt};
    use mongodb::{bson::doc, options::ClientOptions};
    use tokio::select;
    use tokio::sync::mpsc;
    use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
    use tokio_util::{sync::CancellationToken, task::TaskTracker};
    use tracing::{debug, info, Level};
    const MONGO_NAME: &'static str = "artcord_test";
    const MONGO_URL: &'static str = "mongodb://root:U2L63zXot4n5@localhost:27017";

    struct Client {
        key: u128,
        client_tx: mpsc::Sender<(u128, ClientMsg)>,
        //client_recv_tx: mpsc::Sender<(u128, ClientMsg)>,
        //server_send_tx: mpsc::Sender<(u128, ServerMsg)>,
        server_rx: mpsc::Receiver<(u128, ServerMsg)>,
    }

    impl Client {
        pub fn new() -> (
            Self,
            mpsc::Sender<(u128, ServerMsg)>,
            mpsc::Receiver<(u128, ClientMsg)>,
        ) {
            let (client_tx, client_rx) = mpsc::channel::<(u128, ClientMsg)>(1);
            let (server_tx, server_rx) = mpsc::channel::<(u128, ServerMsg)>(1);

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

        pub async fn send(&mut self, msg: ClientMsg) {
            self.client_tx.send((self.key, msg)).await;
        }

        pub async fn recv(&mut self) -> ServerMsg {
            let (_, msg) = self.server_rx.recv().await.unwrap();
            msg
        }
    }

    #[tokio::test]
    async fn ws_test() {
        tracing_subscriber::fmt()
            .event_format(
                tracing_subscriber::fmt::format()
                    .with_file(true)
                    .with_line_number(true),
            )
            .with_env_filter(tracing_subscriber::EnvFilter::from_str("artcord=trace").unwrap())
            .try_init()
            .unwrap();

        drop_db(MONGO_NAME, MONGO_URL).await;
        let db = DB::new(MONGO_NAME, MONGO_URL).await;
        let db = Arc::new(db);

        let task_tracker = TaskTracker::new();
        let cancelation_token = CancellationToken::new();
        let threshold = WsThreshold {
            ws_app_threshold: Threshold::new_const(10, TimeDelta::try_minutes(1)),
            ws_app_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
            ws_app_threshold_range: 5,

            ws_stat_threshold: Threshold::new_const(10, TimeDelta::try_minutes(1)),
            ws_stat_ban_duration: match TimeDelta::try_minutes(1) {
                Some(delta) => delta,
                None => panic!("invalid delta"),
            },
        };

        let (web_sockets_handle, web_sockets_channel) = create_ws(
            task_tracker.clone(),
            cancelation_token.clone(),
            "0.0.0.0:3420",
            &threshold,
            db.clone(),
        )
        .await;

        let mut client = client(task_tracker.clone(), cancelation_token.clone()).await;

        for _ in 0..20 {
            client
                .send(ClientMsg::WsStatsWithPagination {
                    page: 0,
                    amount: 10,
                })
                .await;
            let _ = client.recv().await;
            //debug!("recv: {:#?}", msg);
        }

        client
            .send(ClientMsg::WsStatsWithPagination {
                page: 0,
                amount: 10,
            })
            .await;
        let msg = client.recv().await;
        //ServerMsg::
        let result = matches!(msg, ServerMsg::Reset);

        //assert!();

        info!("exiting...");
        cancelation_token.cancel();
        web_sockets_handle.await.unwrap();
        task_tracker.close();
        task_tracker.wait().await;
    }

    async fn client(task_tracker: TaskTracker, cancellation_token: CancellationToken) -> Client {
        let (channel, server_tx, mut client_rx) = Client::new();
        // let (client_send_tx, mut client_recv_tx) = mpsc::channel::<(u128, ClientMsg)>(1);
        // let (server_send_tx, mut server_recv_tx) = mpsc::channel::<(u128, ServerMsg)>(1);

        task_tracker.spawn(async move {
            let url = url::Url::parse("ws://localhost:3420").unwrap();
            let con = connect_async(url).await;
            let (ws_stream, res) = con.unwrap();
            let (mut write, mut read) = ws_stream.split();

            loop {
                select! {
                    msg = client_rx.recv() => {
                        on_client_recv(&mut write, msg.unwrap()).await;
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
        (key, msg): (u128, ClientMsg),
    ) {
        let bytes = ClientMsg::as_vec(&(key, msg)).unwrap();
        write.send(Message::Binary(bytes)).await.unwrap();
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
}
