use artcord_leptos_web_sockets::WsPackage;
use artcord_leptos_web_sockets::WsRouteKey;
use artcord_state::message::client_msg::ClientMsg;
use artcord_state::message::prod_perm_key::ProdMsgPermKey;
use artcord_state::message::server_msg::ServerMsg;
use futures::pin_mut;
use futures::TryStreamExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::task;

use futures::future;
use futures::SinkExt;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{trace, error};

pub mod ws_route;

pub async fn create_websockets() -> Result<(), String> {
    let addr = String::from("0.0.0.0:3420");
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);
    while let Ok((stream, _)) = listener.accept().await {
        task::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("Connected streams should have a peer address.");
    println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred.");

    println!("New WebSocket connection: {}", addr);

    let (tx, mut rx) = mpsc::channel::<Message>(1000);
    let (mut write, read) = ws_stream.split();
    //let tx = Arc::new(tx);

    let read = read.try_for_each_concurrent(1000, {
        let tx = tx.clone();
        move |msgclient_msg| {
            let tx = tx.clone();
            async move {
                if let Message::Binary(msgclient_msg) = msgclient_msg {
                    let client_msg = ClientMsg::from_bytes(&msgclient_msg);
                    match client_msg {
                        Ok(client_msg) => {
                            println!("SOME MSG: {:?}", &client_msg);
                            let key = client_msg.key;
                            let data = client_msg.data;

                            let server_msg = match data {
                                ClientMsg::GalleryInit { amount, from } => {
                                    let server_package = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
                                        key,
                                        data: ServerMsg::None,
                                    };
                                    trace!("ws: sending: {:?}", &server_package);
                                    ServerMsg::as_bytes(server_package)
                                }
                                _ => {
                                    let server_package = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
                                        key,
                                        data: ServerMsg::None,
                                    };
                                    trace!("ws: sending: {:?}", &server_package);
                                    ServerMsg::as_bytes(server_package)
                                }
                            };
                           
                            match server_msg {
                                Ok(server_msg) => {
                                    let server_msg = Message::binary(server_msg);
                                    let send_result = tx.send(server_msg).await;
                                    match send_result {
                                        Ok(_) => {
                                        }
                                        Err(err) => {
                                            error!("ws: sending server msg error: {}", err);
                                        }
                                    }
                                }
                                Err(err) => {
                                    error!("ws: server msg serialization error: {}", err);
                                }
                            }
                           
                        }
                        Err(err) => {
                            error!(
                                "ws: client msg serialization error: {}",
                                err
                            );
                            let reset_package = WsPackage::<u128, ProdMsgPermKey, ServerMsg> {
                                key: WsRouteKey::Perm(ProdMsgPermKey::Reset),
                                data: ServerMsg::Reset,
                            };
                            trace!("ws: sending: {:?}", &reset_package);
                            let bytes = ServerMsg::as_bytes(reset_package);
                            match bytes {
                                Ok(bytes) => {
                                    let server_msg = Message::binary(bytes);
                                    let send_result = tx.send(server_msg).await;
                                    match send_result {
                                        Ok(_) => {
                                        }
                                        Err(err) => {
                                            error!("ws: sending reset msg error: {}", err);
                                        }
                                    }
                                }
                                Err(err) => {
                                    error!("ws: reset msg serialization error: {}", err);
                                }
                            }
                            // let Ok(bytes) = bytes else {
                            //     println!("Failed to serialize server msg: {}", bytes.err().unwrap());
                            //     return Ok(());
                            // };
                            //Message::Ping(vec![]);
                            
                            
                        }
                    }
                }
                //write.send(server_msg);

                Ok(())
            }
        }
    });

    //let write = rx.forward(write);

    let write = async move {
        while let Some(msg) = rx.recv().await {
            write.send(msg).await.unwrap();
        }
    };

    pin_mut!(read, write);

    future::select(read, write).await;
    // read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
    //     .forward(write)
    //     .await
    //     .expect("Failed to forward message");

    println!("ho ho ho ho");
}
