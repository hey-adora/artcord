

use artcord_state::message::client_msg::ClientMsg;
use artcord_state::message::server_msg::ServerMsg;
use futures::pin_mut;
use futures::TryStreamExt;
use tokio::net::TcpListener;
use tokio::task;
use tokio::net::TcpStream;

use futures::StreamExt;
use futures::future;
use tokio_tungstenite::tungstenite::Message;
use tokio::sync::mpsc;
use futures::SinkExt;

pub async fn create_websockets() -> Result<(), String> {
    let addr = String::from("127.0.0.1:3420");
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
                    let Ok((id, client_msg)) = client_msg else {
                        println!(
                            "Failed to convert bytes to client msg: {}",
                            client_msg.err().unwrap()
                        );
                        let bytes = ServerMsg::Reset.as_bytes(0);
                        let Ok(bytes) = bytes else {
                            println!("Failed to serialize server msg: {}", bytes.err().unwrap());
                            return Ok(());
                        };
                        let server_msg = Message::binary(bytes);
                        tx.send(server_msg).await.unwrap();
                        return Ok(());
                    };

                    println!("SOME MSG: {} {:?}", id, &client_msg);
                    let server_msg = ServerMsg::None.as_bytes(id).unwrap();
                    let server_msg = Message::binary(server_msg);
                    tx.send(server_msg).await.unwrap();
                }
                //write.send(server_msg);
        
                Ok(())
            }
        }
    });

    
    //let write = rx.forward(write);
    
    let write  = async move {
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