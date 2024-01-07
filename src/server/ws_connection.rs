use actix_web_actors::ws::{self, CloseCode, CloseReason, ProtocolError};
use actix::{Actor, Addr, AsyncContext, Handler, Recipient, StreamHandler};
use actix_web::web::Bytes;
use crate::server::client_msg::ClientMsg;
use crate::server::create_server::ServerState;
use crate::server::server_msg::ServerMsg;

pub struct WsConnection {
    pub id: uuid::Uuid,
    pub server_state: ServerState
}

pub struct VecActor(pub Vec<u8>);

impl actix::Message for VecActor {
    type Result = ();
}


struct ByteActor(pub Bytes);

impl actix::Message for ByteActor {
    type Result = ();
}


impl Actor for WsConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let sessions = self.server_state.sessions.write();
        let Ok(mut sessions) = sessions else {
            let error = sessions.err().unwrap();
            println!("Locking WS sessions error: {}", error);
            ctx.close(Some(CloseReason::from(CloseCode::Error)));
            return;
        };
        sessions.insert(self.id, ctx.address());
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let sessions = self.server_state.sessions.write();
        let Ok(mut sessions) = sessions else {
            let error = sessions.err().unwrap();
            println!("Locking WS sessions error: {}", error);
            ctx.close(Some(CloseReason::from(CloseCode::Error)));
            return;
        };
        sessions.remove(&self.id);
    }
}


impl Handler<VecActor> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: VecActor, ctx: &mut Self::Context) -> Self::Result {
        ctx.binary(msg.0);
    }
}

impl Handler<ByteActor> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: ByteActor, ctx: &mut Self::Context) -> () {
        let db = self.server_state.db.clone();
        let recipient: Recipient<_> = ctx.address().recipient();
        let fut = async move {
            let client_msg = ClientMsg::from_bytes(&msg.0.to_vec());
            let Ok(client_msg) = client_msg else {
                println!("Failed to convert bytes to client msg: {}", client_msg.err().unwrap());
                let bytes = rkyv::to_bytes::<_, 256>(&ServerMsg::Reset);
                let Ok(bytes) = bytes else {
                    println!("Failed to serialize serevr msg: {}", bytes.err().unwrap());
                    return;
                };
                recipient.do_send(VecActor(bytes.into_vec()));
                return;
            };
            let server_msg: Result<ServerMsg, mongodb::error::Error> = match client_msg {
                ClientMsg::GalleryInit { amount, from} => {
                    db.img_aggregate_gallery(amount, from).await
                    // MyWs::gallery_handler(db, amount, from).await
                },
                ClientMsg::UserGalleryInit {amount, from, user_id} => {
                    db.img_aggregate_user_gallery(amount, from, &user_id).await
                },
                ClientMsg::User {user_id} => {
                    db.user_find_one(&user_id).await
                }
            };

            let Ok(server_msg) = server_msg else {
                println!("Failed to create server msg: {}", server_msg.err().unwrap());
                return;
            };

            let bytes = rkyv::to_bytes::<_, 256>(&server_msg);
            let Ok(bytes) = bytes else {
                println!("Failed to serialize server msg: {}", bytes.err().unwrap());
                return;
            };

            recipient.do_send(VecActor(bytes.into_vec()));
        };
        let fut = actix::fut::wrap_future::<_, Self>(fut);
        let _a = ctx.spawn(fut);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {

    fn handle(&mut self, msg: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg)
            }
            Ok(ws::Message::Text(_text)) => {
            }
            Ok(ws::Message::Binary(bytes)) => {
                ctx.address().do_send(ByteActor(bytes));
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason)
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
            _ => {
                println!("BOOOM");
            }
        }
    }
}