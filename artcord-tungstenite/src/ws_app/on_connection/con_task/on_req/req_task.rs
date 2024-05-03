use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::{ClientMsg, WsPath};
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::prod_server_msg::ServerMsg;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, trace};

use crate::ws_app::on_connection::con_task::on_req::req_task::res::ws_stats_ranged::ws_stats_ranged;
//use crate::ws_app::on_connection::con_task::on_req::req_task::res::ws_stats_first_page::ws_stats_first_page;
use crate::ws_app::on_connection::con_task::on_req::req_task::res::ws_stats_total_count::ws_stats_total_count;
use crate::ws_app::on_connection::con_task::on_req::req_task::res::ws_stats_with_paginatoin::ws_stats_with_pagination;
use crate::ws_app::on_connection::con_task::ConMsg;
use crate::ws_app::ws_statistic::AdminConStatMsg;
use crate::ws_app::WsResError;

use self::res::live_ws_stats::live_ws_stats;
use self::res::main_gallery::ws_handle_main_gallery;
use self::res::user::ws_handle_user;
use self::res::user_gallery::ws_handle_user_gallery;
use self::res::ws_stats_paged::ws_stats_paged;

pub mod res;

pub async fn req_task(
    client_msg: Message,
    db: Arc<DB>,
    connection_task_tx: mpsc::Sender<ConMsg>,
    admin_ws_stats_tx: mpsc::Sender<AdminConStatMsg>,
    connection_key: String,
    addr: SocketAddr,
) {
    let user_task_result = async {
        // let client_msg = client_msg?;
        let client_msg: Result<Vec<u8>, WsResError> = match client_msg {
            Message::Binary(client_msg) => Ok(client_msg),
            client_msg => Err(WsResError::InvalidClientMsg(client_msg)),
        };

        let client_msg = ClientMsg::from_bytes(&client_msg?)?;
        let res_key: WsRouteKey = client_msg.0;
        let data = client_msg.1;

        trace!("recv: {:#?}", data);

        // sleep(Duration::from_secs(5)).await;

        let get_response_data = async {
            if let ClientMsg::LiveWsStats(listener_state) = data {
                return live_ws_stats(
                    db,
                    listener_state,
                    connection_key,
                    res_key,
                    addr,
                    &connection_task_tx,
                    admin_ws_stats_tx,
                )
                .await;
            }
            // let response_data: Option<Result<Option<ServerMsg>, WsResError>> = match data {
            //     ClientMsg::LiveWsStats(listener_state) => Some(
            //         live_ws_stats(
            //             db.clone(),
            //             listener_state,
            //             connection_key.clone(),
            //             res_key.clone(),
            //             addr,
            //             &connection_task_tx,
            //             admin_ws_stats_tx.clone(),
            //         )
            //         .await,
            //     ),
            //     _ => None,
            // };
            //
            // if let Some(res) = response_data {
            //     return res;
            // }

            admin_ws_stats_tx
                .send(AdminConStatMsg::Inc {
                    connection_key: connection_key.clone(),
                    path: WsPath::from(&data),
                })
                .await?;

            let response_data: Result<Option<ServerMsg>, WsResError> = match data {
                ClientMsg::WsStatsTotalCount { from } => ws_stats_total_count(db, from).await,
                ClientMsg::WsStatsRange { from, to } => ws_stats_ranged(db, from, to).await,
                //ClientMsg::WsStatsFirstPage {  amount } => ws_stats_first_page(db, amount).await,
                ClientMsg::WsStatsPaged { page, amount, from } => ws_stats_paged(db, page, amount, from).await,
                ClientMsg::WsStatsWithPagination { page, amount } => ws_stats_with_pagination(db, page, amount).await,
                ClientMsg::LiveWsStats(listener_state) => {
                    live_ws_stats(
                        db,
                        listener_state,
                        connection_key,
                        res_key,
                        addr,
                        &connection_task_tx,
                        admin_ws_stats_tx,
                    )
                    .await
                }
                ClientMsg::User { user_id } => ws_handle_user(db, user_id).await,
                ClientMsg::UserGalleryInit {
                    amount,
                    from,
                    user_id,
                } => ws_handle_user_gallery(db, amount, from, user_id).await,
                ClientMsg::GalleryInit { amount, from } => {
                    ws_handle_main_gallery(db, amount, from).await
                }
                _ => Ok(Some(ServerMsg::NotImplemented)),
            };
            response_data
        };

        // a

        let response_data = get_response_data.await;

        let response_data = response_data
            .inspect_err(|err| error!("reponse error: {:#?}", err))
            .unwrap_or(Some(ServerMsg::Error));
        let Some(response_data) = response_data else {
            trace!("not sending anything back from request handle task");
            return Ok(());
        };
        let response: WsPackage<ServerMsg> = (res_key, response_data);
        #[cfg(feature = "development")]
        {
            let mut output = format!("{:?}", &response);
            output.truncate(100);
            trace!("sent: {}", output);
        }
        let response = ServerMsg::as_bytes(response)?;
        let response = Message::Binary(response);
        connection_task_tx.send(ConMsg::Send(response)).await?;

        // Ok::<Message, WsUserTaskError>(Message::Binary(bytes))
        Ok::<(), WsResError>(())
    }
    .await;
    if let Err(err) = user_task_result {
        debug!("res error: {}", &err);
        let send_result = connection_task_tx
            .send(ConMsg::Send(Message::Close(Some(CloseFrame {
                code: CloseCode::Error,
                reason: Cow::from("corrupted"),
            }))))
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
