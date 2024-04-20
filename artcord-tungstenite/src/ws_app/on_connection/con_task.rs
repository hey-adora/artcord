use crate::ws_app::on_connection::con_task::on_msg::on_msg;
use crate::ws_app::on_connection::con_task::on_req::on_req;
use crate::ws_app::ws_statistic::AdminConStatMsg;
use crate::ws_app::WsAppMsg;
use artcord_mongodb::database::DB;
use futures::StreamExt;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{error, trace};

pub mod on_msg;
pub mod on_req;

pub enum ConMsg {
    Send(Message),
    Stop,
}

pub async fn con_task(
    stream: TcpStream,
    cancellation_token: CancellationToken,
    db: Arc<DB>,
    ws_tx: mpsc::Sender<WsAppMsg>,
    ip: IpAddr,
    addr: SocketAddr,
    admin_ws_stats_tx: mpsc::Sender<AdminConStatMsg>,
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
    // ws_stream.
    trace!("con accepted");
    let con_id = uuid::Uuid::new_v4().to_string();
    // let Ok(ws_stream) = ws_stream else {
    //     return;
    // };

    let (connection_task_tx, mut connection_task_rx) = mpsc::channel::<ConMsg>(1);
    // let (client_out_handler, mut client_out_handle) = mpsc::channel::<Message>(10);
    let (mut client_out, mut client_in) = ws_stream.split();
    let user_task_tracker = TaskTracker::new();

    let send_result = admin_ws_stats_tx
        .send(AdminConStatMsg::AddTrack {
            connection_key: con_id.clone(),
            tx: connection_task_tx.clone(),
            addr: addr.to_string(),
        })
        .await;
    if let Err(err) = send_result {
        error!("error adding track: {}", err);
    }

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
                let exit = on_req(result, &user_task_tracker, &db, &connection_task_tx, &admin_ws_stats_tx, &con_id, &addr).await;
                if exit {
                    break;
                }
            },

            // result = client_out_handle.recv() => {
            //     trace!("write finished");
            //     let exit = request_write_task(&mut client_out, result).await;
            //     if exit {
            //         break;
            //     }
            // },

            result = connection_task_rx.recv() => {
                let exit = on_msg(result, &mut client_out).await;
                if exit {
                    break;
                }
            },

            _ = cancellation_token.cancelled() => {
                break;
            }
        }
    }

    let send_result = admin_ws_stats_tx
        .send(AdminConStatMsg::RemoveRecv {
            connection_key: con_id.clone(),
        })
        .await;
    if let Err(err) = send_result {
        error!("error removing recv: {}", err);
    }

    user_task_tracker.close();
    user_task_tracker.wait().await;
    let send_result = ws_tx.send(WsAppMsg::Disconnected(ip)).await;
    if let Err(err) = send_result {
        error!("failed to send disconnect to ws: {}", err);
    }
    trace!("disconnected");
}
