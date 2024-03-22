use std::net::SocketAddr;
use std::sync::Arc;

use artcord_leptos_web_sockets::WsPackage;
use artcord_leptos_web_sockets::WsRouteKey;
use artcord_mongodb::database::DB;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::prod_server_msg::ServerMsg;
use futures::pin_mut;
use futures::TryStreamExt;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::task;

use futures::future;
use futures::SinkExt;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::Instrument;
use tracing::{error, trace};
use ws_route::ws_main_gallery::WsHandleMainGalleryError;
use ws_route::ws_user::WsHandleUserError;
use ws_route::ws_user_gallery::WsHandleUserGalleryError;

use crate::ws_route::ws_main_gallery::ws_handle_main_gallery;
use crate::ws_route::ws_user::ws_handle_user;
use crate::ws_route::ws_user_gallery::ws_handle_user_gallery;

pub mod ws_route;

pub async fn create_websockets(db: Arc<DB>) -> Result<(), String> {
    let ws_addr = String::from("0.0.0.0:3420");
    let try_socket = TcpListener::bind(&ws_addr).await;
    let listener = try_socket.expect("Failed to bind");
    trace!("ws({}): started", &ws_addr);
    while let Ok((stream, _)) = listener.accept().await {
        let Ok(user_addr) = stream
            .peer_addr()
            .inspect_err(|err| error!("failed to get peer addr: {}", err))
        else {
            continue;
        };

        task::spawn(accept_connection(user_addr, stream, db.clone()).instrument(
            tracing::trace_span!("ws", "{}-{}", ws_addr, user_addr.to_string()),
        ));
    }

    Ok(())
}

async fn accept_connection(addr: SocketAddr, stream: TcpStream, db: Arc<DB>) {
    trace!("connecting...");

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred.");

    trace!("connected");

    let (send, mut recv) = mpsc::channel::<Message>(1000);
    let (mut write, read) = ws_stream.split();

    let read = read.try_for_each_concurrent(1000, {
        move |client_msg| {
            let send = send.clone();
            let db = db.clone();
            async move {
                let _ = response_handler(db, send, client_msg)
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

    trace!("disconnected");
}

async fn response_handler(
    db: Arc<DB>,
    send: mpsc::Sender<Message>,
    client_msg: Message,
) -> Result<(), WsResponseHandlerError> {
    if let Message::Binary(msgclient_msg) = client_msg {
        let client_msg = ClientMsg::from_bytes(&msgclient_msg)?;

        trace!("received: {:?}", &client_msg);
        let key: WsRouteKey<u128, ProdMsgPermKey> = client_msg.key;
        let data = client_msg.data;

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

        #[cfg(debug_assertions)]
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
