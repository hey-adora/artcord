use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::prod_server_msg::ServerMsg;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, trace};

use crate::ws_app::on_connection::con_task::ConMsg;
use crate::ws_app::ws_statistic::WsThrottleListenerMsg;
use crate::ws_app::WsResError;

use self::res::admin_statistics::ws_hadnle_admin_throttle;
use self::res::main_gallery::ws_handle_main_gallery;
use self::res::user::ws_handle_user;
use self::res::user_gallery::ws_handle_user_gallery;

pub mod res;

pub async fn req_task(
    client_msg: Message,
    db: Arc<DB>,
    connection_task_tx: mpsc::Sender<ConMsg>,
    throttle_tx: mpsc::Sender<WsThrottleListenerMsg>,
    connection_key: uuid::Uuid,
    addr: SocketAddr,
) {
    let user_task_result = async {
        // let client_msg = client_msg?;
        let client_msg: Result<Vec<u8>, WsResError> = match client_msg {
            Message::Binary(client_msg) => Ok(client_msg),
            client_msg => Err(WsResError::InvalidClientMsg(client_msg)),
        };

        let client_msg = ClientMsg::from_bytes(&client_msg?)?;
        let res_key: WsRouteKey<u128, ProdMsgPermKey> = client_msg.key;
        let data = client_msg.data;

        // sleep(Duration::from_secs(5)).await;

        let response_data: Result<Option<ServerMsg>, WsResError> = match data {
            ClientMsg::AdminThrottleListenerToggle(listener_state) => {
                ws_hadnle_admin_throttle(
                    db,
                    listener_state,
                    connection_key,
                    res_key.clone(),
                    addr,
                    &connection_task_tx,
                    throttle_tx,
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
        let response_data = response_data
            .inspect_err(|err| error!("reponse error: {:#?}", err))
            .unwrap_or(Some(ServerMsg::Error));
        let Some(response_data) = response_data else {
            trace!("not sending anything back from request handle task");
            return Ok(());
        };
        let response = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
            key: res_key,
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
