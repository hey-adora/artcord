use std::borrow::Cow;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::{global, backend};
use enum_index::EnumIndex;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, trace};

use crate::ws::con::req::res::ResErr;


pub mod res;
pub mod stats;

pub async fn req_task(
    client_msg: Message,
    db: Arc<DB>,
    con_tx: mpsc::Sender<backend::ConMsg>,
    ws_tx: mpsc::Sender<backend::WsMsg>,
    connection_key: global::TempConIdType,
    addr: SocketAddr,
    ip: IpAddr,
    //get_threshold: impl global::ClientThresholdMiddleware,
) {
    trace!("started");
    let user_task_result = async {
        
        // let client_msg = client_msg?;
        let client_msg: Result<Vec<u8>, ResErr> = match client_msg {
            Message::Binary(client_msg) => Ok(client_msg),
            client_msg => Err(ResErr::InvalidClientMsg(client_msg)),
        };

        let client_msg = global::ClientMsg::from_bytes(&client_msg?)?;
        let res_key: WsRouteKey = client_msg.0;
        let data: global::ClientMsg = client_msg.1;
        let path_index = data.enum_index();
        //let path_throttle = get_threshold.get_threshold(&data);

        trace!("recv: {:#?}", data);

        let (allow_tx, allow_rx) = oneshot::channel();
        con_tx
            .send(backend::ConMsg::CheckThrottle {
                path: path_index,
                //block_threshold: path_throttle,
                allow_tx,
            })
            .await?;

        let allow = allow_rx.await?;
        trace!("allow: {}", allow);

        let get_response_data = async {
            if !allow {
                return Ok(Some(global::ServerMsg::TooManyRequests));
            }

            let response_data: Result<Option<global::ServerMsg>, ResErr> = match data {
                global::ClientMsg::BanIp { ip, date, reason } => {
                    //let (done_tx, done_rx) = oneshot::channel::<()>();
                    ws_tx.send(backend::WsMsg::Ban { ip, date, reason }).await?;
                    //done_rx.await?;
                    Ok(None)
                }
                global::ClientMsg::WsStatsTotalCount { from } => res::ws_stats::total_count(db, from).await,
                global::ClientMsg::WsStatsGraph {
                    from,
                    to,
                    unique_ip,
                } => res::ws_stats::graph(db, from, to, unique_ip).await,
                //ClientMsg::WsStatsFirstPage {  amount } => ws_stats_first_page(db, amount).await,
                global::ClientMsg::WsStatsPaged { page, amount, from } => {
                    res::ws_stats::paged(db, page, amount, from).await
                }
                global::ClientMsg::WsStatsWithPagination { page, amount } => {
                    res::ws_stats::pagination(db, page, amount).await
                }
                global::ClientMsg::LiveWsThrottleCache(listener_state) => {
                    res::ws_throttle::ws_throttle_cached(
                        db,
                        listener_state,
                        connection_key,
                        res_key,
                        &con_tx,
                        &ws_tx,
                    )
                    .await
                }
                global::ClientMsg::LiveWsStats(listener_state) => {
                    res::ws_stats::live(
                        listener_state,
                        &con_tx,
                        res_key
                    )
                    .await
                }
                global::ClientMsg::User { user_id } => res::user::user(db, user_id).await,
                global::ClientMsg::UserGalleryInit {
                    amount,
                    from,
                    user_id,
                } => res::user::user_gallery(db, amount, from, user_id).await,
                global::ClientMsg::GalleryInit { amount, from } => {
                    res::gallery::gallery(db, amount, from).await
                }
                _ => Ok(Some(global::ServerMsg::NotImplemented)),
            };
            response_data
        };

        let response_data = get_response_data.await;

        let response_data = response_data
            .inspect_err(|err| error!("reponse error: {:#?}", err))
            .unwrap_or(Some(global::ServerMsg::Error));
        let Some(response_data) = response_data else {
            trace!("not sending anything back from request handle task");
            return Ok(());
        };
        let response: WsPackage<global::ServerMsg> = (res_key, response_data);
        #[cfg(feature = "development")]
        {
            trace!("sent res: {:#?}", response);
        }
        let response = global::ServerMsg::as_bytes(response)?;
        let response = Message::Binary(response);
        con_tx.send(backend::ConMsg::Send(response)).await?;

        Ok::<(), ResErr>(())
    }
    .await;
    
    if let Err(err) = user_task_result {
        debug!("res error: {}", &err);
        let send_result = con_tx
            .send(backend::ConMsg::Send(Message::Close(Some(CloseFrame {
                code: CloseCode::Error,
                reason: Cow::from("corrupted"),
            }))))
            .await;
        if let Err(err) = send_result {
            error!("failed to send close signal: {}", err);
        }
        // connection_task_tx.send(WsConnectionMsg::Stop);
    }

    trace!("ended");
    
}
