use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
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
use tracing::instrument;
use tracing::Instrument;
use tracing::{error, trace};

use crate::user_task::UserTask;
use crate::ws_route::ws_admin_throttle::ws_hadnle_admin_throttle;
use crate::ws_route::ws_main_gallery::ws_handle_main_gallery;
use crate::ws_route::ws_statistics;
use crate::ws_route::ws_statistics::ws_statistics;
use crate::ws_route::ws_user::ws_handle_user;
use crate::ws_route::ws_user_gallery::ws_handle_user_gallery;

pub enum WsAppMsg {
    Stop,
}

// pub struct WsAppDTO {
//     pub ws_send: mpsc::Sender<WsAppMsg>,
//     pub addr: &str,
// }

// pub struct WsUserDTO {
//     pub stuff: String,
// }
//
// pub struct WsReqDTO {
//     pub stuff: String,
// }
//
// pub struct WsAppState {
//     pub app: WsAppDTO,
//     pub user: WsUserDTO,
//     pub req: WsReqDTO,
// }

pub struct WsApp {}
// "0.0.0.0:3420"
pub async fn create_ws(
    task_tracker: TaskTracker,
    cancellation_token: CancellationToken,
    addr: &str,
    db: Arc<DB>,
) -> (JoinHandle<()>, mpsc::Sender<WsAppMsg>) {
    let ws_addr = String::from(addr);
    let try_socket = TcpListener::bind(&ws_addr).await;
    let listener = try_socket.expect("Failed to bind");
    trace!("ws({}): started", &ws_addr);

    let (ws_send, mut ws_recv) = mpsc::channel::<WsAppMsg>(1);

    let handle: JoinHandle<()> = task_tracker.spawn({
        let task_tracker = task_tracker.clone();

        async move {
            // run taks
            loop {
                // let handle_con = async {};
                //
                // let handle_msg = async {
                //     let result = ws_recv.recv();
                //     // .....
                // };
                //
                // let (stream, user_addr) = match listener.accept().await {
                //     Ok(result) => result,
                //     Err(err) => {
                //         trace!("ws({}): error accepting connection: {}", &ws_addr, err);
                //         return;
                //     }
                // };

                select! {
                    con = listener.accept() => {
                        on_connection(con, &cancellation_token, &db, &task_tracker, &ws_addr).await;
                    },

                    ws_msg = ws_recv.recv() => {
                        let exit = on_ws_msg(ws_msg).await;
                        if exit {
                            break;
                        }
                    },

                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                }
            }
        }
    });

    (handle, ws_send)
    // let mut throttle = Throttle::new();
}

pub async fn on_connection(
    // listener: TcpListener,
    // ws_addr: &str,
    con: Result<(TcpStream, SocketAddr), io::Error>,
    cancellation_token: &CancellationToken,
    db: &Arc<DB>,
    task_tracker: &TaskTracker,
    ws_addr: &str,
) {
    // debug!("HELLO ONE");
    // let Some(user_throttle_stats) = throttle.maybe_connect_to_ws(user_addr.ip()).await
    // else {
    //     debug!("HELLO TWO");
    //     continue;
    // };
    // let ws_connection_count = *user_throttle_stats.ws_connection_count.read().await;

    // debug!("con count: {}", ws_connection_count);

    // task_tracker.spawn(accept_connection(user_addr, stream, db.clone()).instrument(
    //     tracing::trace_span!("ws", "{}-{}", ws_addr, user_addr.to_string()),
    // ));
    let (stream, user_addr) = match con {
        Ok(result) => result,
        Err(err) => {
            trace!("ws({}): error accepting connection: {}", &ws_addr, err);
            return;
        }
    };
    task_tracker.spawn(
        connection_task(stream, cancellation_token.clone(), db.clone()).instrument(
            tracing::trace_span!("ws", "{}-{}", ws_addr, user_addr.to_string()),
        ),
    );
}

pub async fn on_ws_msg(msg: Option<WsAppMsg>) -> bool {
    let Some(msg) = msg else {
        trace!("ws_recv channel closed");
        return true;
    };
    match msg {
        WsAppMsg::Stop => {
            return true;
        }
    }
    false
}

pub enum WsConnectionMsg {
    Stop,
}

pub async fn connection_task(
    stream: TcpStream,
    cancellation_token: CancellationToken,
    db: Arc<DB>,
) {
    trace!("task spawned!");
    let ws_stream = tokio_tungstenite::accept_async(stream).await;
    // .expect("Error during the websocket handshake occurred.");
    let ws_stream = match ws_stream {
        Ok(ws_stream) => ws_stream,
        Err(err) => {
            trace!("ws_error: {}", err);
            return;
        }
    };
    trace!("con accepted");
    // let Ok(ws_stream) = ws_stream else {
    //     return;
    // };

    let (connection_task_tx, mut connection_task_rx) = mpsc::channel::<WsConnectionMsg>(1);
    let (client_out_handler, mut client_out_handle) = mpsc::channel::<Message>(10);
    let (mut client_out, mut client_in) = ws_stream.split();
    let user_task_tracker = TaskTracker::new();

    // let read = async {};

    // let write = async {
    //     while let Some(msg) = .await {
    //         let send_result = client_out.send(msg).await;
    //         if let Err(send_result) = send_result {
    //             error!("send error: {}", send_result);
    //             return;
    //         }
    //     }
    // };

    // pin_mut!(read, write);

    loop {
        select! {
            result = client_in.next() => {
                trace!("read finished");
                let exit = request_read_task(result, &user_task_tracker, &db, &client_out_handler, &connection_task_tx).await;
                if exit {
                    break;
                }
            },

            result = client_out_handle.recv() => {
                trace!("write finished");
                let exit = request_write_task(&mut client_out, result).await;
                if exit {
                    break;
                }
            },

            result = connection_task_rx.recv() => {
                let exit = on_connection_msg(result).await;
                if exit {
                    break;
                }
            },

            _ = cancellation_token.cancelled() => {
                break;
            }
        }
    }

    user_task_tracker.close();
    user_task_tracker.wait().await;
    trace!("disconnected");
}

pub async fn on_connection_msg(msg: Option<WsConnectionMsg>) -> bool {
    let Some(msg) = msg else {
        trace!("connection msg channel closed");
        return true;
    };

    match msg {
        WsConnectionMsg::Stop => {
            return true;
        }
    }

    false
}

pub async fn request_write_task(
    client_out: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    msg: Option<Message>,
) -> bool {
    let Some(msg) = msg else {
        trace!("write task channel closed");
        return true;
    };

    let send_result = client_out.send(msg).await;
    if let Err(send_result) = send_result {
        error!("send error: {}", send_result);
        return false;
    }

    false
}

pub async fn request_read_task(
    result: Option<Result<Message, tokio_tungstenite::tungstenite::error::Error>>,
    // mut client_in: SplitStream<WebSocketStream<TcpStream>>,
    user_task_tracker: &TaskTracker,
    db: &Arc<DB>,
    client_out_handler: &mpsc::Sender<Message>,
    connection_task_tx: &mpsc::Sender<WsConnectionMsg>,
) -> bool {
    let Some(result) = result else {
        trace!("read.next() returned None");
        return true;
    };

    let client_msg = match result {
        Ok(result) => result,
        Err(err) => {
            debug!("recv msg error: {}", err);
            return false;
        }
    };

    user_task_tracker.spawn(request_handle_task(
        client_msg,
        db.clone(),
        client_out_handler.clone(),
        connection_task_tx.clone(),
    ));

    false
}

pub async fn request_handle_task(
    client_msg: Message,
    db: Arc<DB>,
    client_out_handler: mpsc::Sender<Message>,
    connection_task_tx: mpsc::Sender<WsConnectionMsg>,
) {
    let user_task_result = async {
        // let client_msg = client_msg?;
        let client_msg: Result<Vec<u8>, WsResError> = match client_msg {
            Message::Binary(client_msg) => Ok(client_msg),
            client_msg => Err(WsResError::InvalidClientMsg),
        };

        let client_msg = ClientMsg::from_bytes(&client_msg?)?;
        let key: WsRouteKey<u128, ProdMsgPermKey> = client_msg.key;
        let data = client_msg.data;

        // sleep(Duration::from_secs(5)).await;

        let response_data: Result<ServerMsg, WsResError> = match data {
            ClientMsg::User { user_id } => ws_handle_user(db, user_id).await,
            ClientMsg::UserGalleryInit {
                amount,
                from,
                user_id,
            } => ws_handle_user_gallery(db, amount, from, user_id).await,
            ClientMsg::GalleryInit { amount, from } => {
                ws_handle_main_gallery(db, amount, from).await
            }
            _ => Ok(ServerMsg::NotImplemented),
        };
        let response_data = response_data
            .inspect_err(|err| error!("reponse error: {:#?}", err))
            .unwrap_or(ServerMsg::Error);
        let response = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
            key,
            data: response_data,
        };
        #[cfg(feature = "development")]
        {
            let mut output = format!("{:?}", &response);
            output.truncate(100);
            trace!("sent: {}", output);
        }
        let response = ServerMsg::as_bytes(response)?;
        let response = Message::Binary(response);
        client_out_handler.send(response).await?;

        // Ok::<Message, WsUserTaskError>(Message::Binary(bytes))
        Ok::<(), WsResError>(())
    }
    .await;
    if let Err(err) = user_task_result {
        debug!("res error: {}", &err);
        let send_result = client_out_handler
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Error,
                reason: Cow::from("corrupted"),
            })))
            .await;
        if let Err(err) = send_result {
            error!("failed to send close signal: {}", err);
        }
        // connection_task_tx.send(WsConnectionMsg::Stop);
    }
    // match user_task_result {
    //     Ok(response) => {
    //         // let send_result = send.send(response).await;
    //         // if let Err(err) = send_result {
    //         //     trace!("sending to client err: {}", err);
    //         // }
    //     }
    //     Err(err) => {
    //         debug!("res error: {}", &err);
    //         // let error_response = ServerMsg::NotImplemented
    //         // let send_result = send.send(response).await;
    //         // if let Err(err) = send_result {
    //         //     trace!("sending to client err: {}", err);
    //         // }
    //     }
    // }
}

// pub async fn start(&self) {
// }

// async fn accept_connection(
//     // user_throttle_task: Arc<ThrottleStats>,
//     addr: SocketAddr,
//     stream: TcpStream,
//     db: Arc<DB>,
// ) {
//     //trace!("connecting...");
//
//     let ws_stream = tokio_tungstenite::accept_async(stream)
//         .await
//         .expect("Error during the websocket handshake occurred.");
//
//     trace!("connected");
//
//     let (send, mut recv) = mpsc::channel::<Message>(10);
//     let (mut write, mut read) = ws_stream.split();
//     let task_tracker = TaskTracker::new();
//     // let mut admin_task_handle = UserTask::new(task_tracker.clone());
//     // let i = time::interval(Duration::from_secs(5));
//     // let is_admin_throttle_listener_active: Arc<Mutex<Option<JoinHandle<()>>>> =
//     //     Arc::new(Mutex::new(None));
//     // let (admin_cancel_send, admin_cancel_recv) = broadcast::channel::<bool>(1);
//     // let adming_throttle_listener_is_closed = oneshot::channel();
//     // let
//
//     let read = {
//         // let admin_task_handle = admin_task_handle.clone();
//         // let user_throttle_task = &user_throttle_task;
//         let task_tracker = &task_tracker;
//         async move {
//             loop {
//                 let result = read.next().await;
//                 let Some(result) = result else {
//                     trace!("read.next() returned None");
//                     break;
//                 };
//                 match result {
//                     Ok(client_msg) => {
//                         // tokio::spawn(future)
//
//                         // debug!("proccesing request count: {}", task_tracker.len());
//                         // if task_tracker.len() > 2 {
//                         //     debug!("WS REQUEST LIMIT REACHED");
//                         //     user_throttle_task.maybe_ban().await;
//                         //     // *user_throttle_task.ws_red_flag.write().await += 1;
//                         //     // *red_flag += 1;
//                         //     break;
//                         // }
//                         //
//                         let send = send.clone();
//                         let db = db.clone();
//                         // handler_tasks.len();
//                         // let user_throttle_task = user_throttle_task.clone();
//                         // let admin_task_handle = admin_task_handle.clone();
//                         let handle_task = {
//                             async move {
//                                 let _ = response_handler(
//                                     // user_throttle_task,
//                                     db, send,
//                                     client_msg,
//                                     // admin_task_handle,
//                                     // task_tracker,
//                                     // is_admin_throttle_listener_active,
//                                     // admin_cancel_recv,
//                                     // admin_cancel_send,
//                                 )
//                                 .await
//                                 .inspect_err(|err| error!("reponse handler error: {:#?}", err));
//                             }
//                         };
//                         task_tracker.spawn(handle_task);
//                     }
//                     Err(err) => {
//                         error!("error receiving message: {}", err);
//                     }
//                 }
//             }
//         }
//     };
//     // let read = read.try_for_each_concurrent(10, {
//     //     |client_msg| {
//     //         let send = send.clone();
//     //         let db = db.clone();
//     //         let user_throttle_task = user_throttle_task.clone();
//     //         async move {
//     //             cfg_if! {
//     //                 if #[cfg(feature = "beta")] {
//     //                     let _ = response_handler_beta(user_throttle_task, db, send, client_msg)
//     //                     .await
//     //                     .inspect_err(|err| error!("reponse handler error: {:#?}", err));
//     //                 } else {
//     //                     let _ = response_handler(user_throttle_task, db, send, client_msg)
//     //                     .await
//     //                     .inspect_err(|err| error!("reponse handler error: {:#?}", err));
//     //                 }
//     //             }
//     //
//     //             Ok(())
//     //         }
//     //     }
//     // });
//
//     let write = async move {
//         while let Some(msg) = recv.recv().await {
//             write.send(msg).await.unwrap();
//         }
//     };
//
//     pin_mut!(read, write);
//
//     select! {
//         _ = read => {
//
//         },
//         _ = write => {
//
//         }
//     }
//
//     // admin_task_handle.stop().await;
//     task_tracker.close();
//     task_tracker.wait().await;
//     // future::select(read, write).await;
//
//     //*user_throttle_task.ws_connection_count.write().await -= 1;
//     // let can_sub = user_throttle_task.ws_connection_count.write().await.checked_sub(1);
//
//     // {
//     //     let mut ws_connection_count = user_throttle_task.ws_connection_count.write().await;
//     //     let can_sub = ws_connection_count.checked_sub(1);
//     //     if let Some(new_ws_connection_count) = can_sub {
//     //         *ws_connection_count = new_ws_connection_count;
//     //     } else {
//     //         error!("throttle: failed to subtract 1");
//     //     }
//     // }
//
//     // trace!(
//     //     "disconnected: {}",
//     //     *user_throttle_task.ws_connection_count.read().await
//     // );
// }
//
// async fn response_handler(
//     // user_throttle_task: Arc<ThrottleStats>,
//     db: Arc<DB>,
//     send: mpsc::Sender<Message>,
//     client_msg: Message,
//     // admin_task: UserTask,
//     // task_tracker: TaskTracker,
//     // is_admin_throttle_listener_active: Arc<Mutex<Option<JoinHandle<()>>>>,
//     // admin_cancel_recv: broadcast::Receiver<bool>,
//     // admin_cancel_send: broadcast::Sender<bool>,
// ) -> Result<(), WsResError> {
//     if let Message::Binary(msgclient_msg) = client_msg {
//         let client_msg = ClientMsg::from_bytes(&msgclient_msg)?;
//
//         trace!("received: {:?}", &client_msg);
//         let key: WsRouteKey<u128, ProdMsgPermKey> = client_msg.key;
//         let data = client_msg.data;
//         let ws_path: WsPath = WsPath::from(&data);
//
//         // user_throttle_task.maybe_sleep(&ws_path).await;
//
//         let server_msg: Result<ServerMsg, WsResError> = match data {
//             // ClientMsg::GalleryInit { amount, from } => ws_handle_main_gallery(db, amount, from)
//             //     .await
//             //     .map(ServerMsg::MainGallery)
//             //     .map_err(WsResponseHandlerError::MainGallery),
//             // ClientMsg::UserGalleryInit {
//             //     amount,
//             //     from,
//             //     user_id,
//             // } => ws_handle_user_gallery(db, amount, from, user_id)
//             //     .await
//             //     .map(ServerMsg::UserGallery)
//             //     .map_err(WsResponseHandlerError::UserGallery),
//             // ClientMsg::User { user_id } => ws_handle_user(db, user_id)
//             //     .await
//             //     .map(ServerMsg::User)
//             //     .map_err(WsResponseHandlerError::User),
//             // ClientMsg::Statistics => ws_statistics(db)
//             //     .await
//             //     .map(ServerMsg::Statistics)
//             //     .map_err(WsResponseHandlerError::Statistics),
//             // ClientMsg::AdminThrottleListenerToggle(state) => ws_hadnle_admin_throttle(
//             //     db, state,
//             //     admin_task,
//             //     // task_tracker,
//             //     // is_admin_throttle_listener_active,
//             //     // admin_cancel_recv,
//             //     // admin_cancel_send,
//             // )
//             // .await
//             // .map(ServerMsg::AdminThrottle)
//             // .map_err(WsResponseHandlerError::AdminThrottle),
//             _ => Ok(ServerMsg::NotImplemented),
//         };
//
//         let server_msg = server_msg
//             .inspect_err(|err| {
//                 error!("reponse error: {:#?}", err);
//             })
//             .unwrap_or(ServerMsg::Error);
//
//         let server_package = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
//             key,
//             data: server_msg,
//         };
//
//         #[cfg(feature = "development")]
//         {
//             let mut output = format!("{:?}", &server_package);
//             output.truncate(100);
//             trace!("sent: {}", output);
//         }
//
//         let bytes = ServerMsg::as_bytes(server_package)?;
//
//         let server_msg = Message::binary(bytes);
//         send.send(server_msg).await?;
//     }
//     Ok(())
// }

#[derive(Error, Debug)]
pub enum WsResError {
    // #[error("Statistics error: {0}")]
    // AdminThrottle(#[from] WsHandleAdminThrottleError),
    //
    // #[error("Statistics error: {0}")]
    // Statistics(#[from] WsStatisticsError),
    //
    // #[error("MainGallery error: {0}")]
    // MainGallery(#[from] WsHandleMainGalleryError),
    //
    // #[error("MainGallery error: {0}")]
    // UserGallery(#[from] WsHandleUserGalleryError),
    //
    // #[error("User error: {0}")]
    // User(#[from] WsHandleUserError),
    #[error("Invalid client msg error")]
    InvalidClientMsg,

    #[error("MainGallery error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Send error: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>),
    // tokio::sync::mpsc::error::SendError<tokio_tungstenite::tungstenite::Message>>>
    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),

    // #[error("Bcrypt error: {0}")]
    // Bcrypt(#[from] bcrypt::BcryptError),
    //
    // #[error("JWT error: {0}")]
    // JWT(#[from] jsonwebtoken::errors::Error),
    #[error("RwLock error: {0}")]
    RwLock(String),
}
