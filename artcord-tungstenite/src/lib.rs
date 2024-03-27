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
use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;
use futures::pin_mut;
use futures::TryStreamExt;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::task;

use futures::future;
use futures::SinkExt;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tracing::debug;
use tracing::instrument;
use tracing::Instrument;
use tracing::{error, trace};
use ws_route::ws_main_gallery::WsHandleMainGalleryError;
use ws_route::ws_user::WsHandleUserError;
use ws_route::ws_user_gallery::WsHandleUserGalleryError;
use tokio::time::Instant;

use crate::ws_route::ws_main_gallery::ws_handle_main_gallery;
use crate::ws_route::ws_user::ws_handle_user;
use crate::ws_route::ws_user_gallery::ws_handle_user_gallery;

pub mod ws_route;

const WS_LIMIT_MAX_CONNECTIONS: u64 = 10;
//const WS_LIMIT_THROTTLE: u64 = 10;

pub struct ThrottleStats {
    ws_connection_count: RwLock<u64>,
    ws_path_count: RwLock<HashMap<WsPath, (u64, Instant)>>,
   // ws_path_interval: RwLock<DateTime<chrono::Utc>>,
    //ws_last_connection: RwLock<u64>,
}

impl ThrottleStats {
    pub fn new() -> Self {
        Self {
            ws_connection_count: RwLock::new(1),
            ws_path_count: RwLock::new(HashMap::new()),
         //   ws_path_interval: RwLock::new(Utc::now())
        }
    }
}

pub async fn create_websockets(db: Arc<DB>) -> Result<(), String> {
    let ws_addr = String::from("0.0.0.0:3420");
    let try_socket = TcpListener::bind(&ws_addr).await;
    let listener = try_socket.expect("Failed to bind");
    trace!("ws({}): started", &ws_addr);

    //let throttle = Arc::new(RwLock::new(HashMap::<SocketAddr, AtomicU64>::new()));
    let mut throttle = HashMap::<IpAddr, Arc<ThrottleStats>>::new();

    loop  {
        let (stream, user_addr) = match listener.accept().await {
            Ok(result) => result,
            Err(err) => {
                error!("ws({}): error accepting connection: {}", &ws_addr, err);
                continue;
            }
        };
        let ip = user_addr.ip();
        let user_throttle_stats = throttle.get(&ip);
        let user_throttle_stats: Arc<ThrottleStats> = if let Some(user_throttle_stats) = user_throttle_stats {
            trace!("ws({}): throttle: stats exist", &ws_addr);
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
            trace!("ws({}): throttle: {} > {}", &ws_addr, count, WS_LIMIT_MAX_CONNECTIONS);
            if count > WS_LIMIT_MAX_CONNECTIONS {
                trace!("ws({}): connection limit reached: {}", &ws_addr, count);
                continue;
            }
            *user_throttle_stats.ws_connection_count.write().await += 1;
            trace!("ws({}): throttle: incremented to: {}", &ws_addr, *user_throttle_stats.ws_connection_count.read().await);
            user_throttle_stats.clone()
        } else {
            trace!("ws({}): throttle: created new", &ws_addr);
            let user_throttle_stats = Arc::new(ThrottleStats::new());
            throttle.insert(ip, user_throttle_stats.clone());
            user_throttle_stats
        };

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

        task::spawn(accept_connection(user_throttle_stats, user_addr, stream, db.clone()).instrument(
            tracing::trace_span!("ws", "{}-{}", ws_addr, user_addr.to_string()),
        ));
    }

    Ok(())
}

async fn accept_connection(user_throttle_task: Arc<ThrottleStats>, addr: SocketAddr, stream: TcpStream, db: Arc<DB>) {
    //trace!("connecting...");

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred.");

    trace!("connected");

    let (send, mut recv) = mpsc::channel::<Message>(1000);
    let (mut write, read) = ws_stream.split();

    let read = read.try_for_each_concurrent(1000, {
         |client_msg| {
            let send = send.clone();
            let db = db.clone();
            let user_throttle_task = user_throttle_task.clone();
            async move {
                let _ = response_handler(user_throttle_task, db, send, client_msg)
                    .await
                    .inspect_err(|err| error!("reponse handler error: {:#?}", err));
                Ok(())
            }
        }
    });

    let write = async move {
        while let Some(msg) = recv.recv().await {
            write.send(msg).await.unwrap();
        }
    };

    pin_mut!(read, write);

    future::select(read, write).await;

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
   
    trace!("disconnected: {}", *user_throttle_task.ws_connection_count.read().await);
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
