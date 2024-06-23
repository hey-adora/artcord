use artcord_state::model::acc::Acc;
use crate::message::server_msg::ServerMsg;
use crate::server::client_msg::{ClientMsg, WsPath};
use crate::server::create_server::{ServerState, TOKEN_SIZE};
use crate::server::registration_invalid::{RegistrationInvalidMsg, BCRYPT_COST};
use crate::server::ws_connection::ws_login::ws_login;
use crate::server::ws_connection::ws_logout::ws_logout;
use crate::server::ws_connection::ws_registration::ws_register;
use actix::{
    Actor, ActorContext, Addr, AsyncContext, ContextFutureSpawner, Handler, Recipient,
    StreamHandler,
};
use actix_web::web::Bytes;
use actix_web_actors::ws::{self, CloseCode, CloseReason, ProtocolError};
use chrono::Utc;
use image::EncodableLayout;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::Rng;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::task::spawn_local;
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub mod ws_login;
mod ws_logout;
pub mod ws_registration;

pub struct WsConnection {
    pub id: uuid::Uuid,
    pub ip: IpAddr,
    pub acc: Arc<RwLock<Option<Acc>>>,
    pub server_state: ServerState,
    pub hb: Instant,
}

impl WsConnection {
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        let sessions = self.server_state.sessions.clone();
        let id = self.id;

        ctx.run_interval(HEARTBEAT_INTERVAL, {
            move |act, ctx| {
                let close_actor: Recipient<CloseActor> = ctx.address().recipient();
                let sessions = sessions.clone();
                let id = id.clone();
                if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                    ctx.stop();

                    spawn_local(async move {
                        let mut sessions = sessions.write().await;
                        sessions.remove(&id);
                    });
                    return;
                }
                ctx.ping(b"");
                //ctx.binary(PING_MSG_BYTES);
            }
        });
    }
}

pub struct HbActor;
impl actix::Message for crate::server::ws_connection::HbActor {
    type Result = ();
}
impl Handler<crate::server::ws_connection::HbActor> for WsConnection {
    type Result = ();

    fn handle(
        &mut self,
        msg: crate::server::ws_connection::HbActor,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        self.hb(ctx)
    }
}

pub struct CloseActor;
impl actix::Message for CloseActor {
    type Result = ();
}
impl Handler<CloseActor> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: CloseActor, ctx: &mut Self::Context) -> Self::Result {
        ctx.close(Some(CloseReason::from(CloseCode::Error)));
    }
}

pub struct VecActor(pub Vec<u8>);

impl actix::Message for VecActor {
    type Result = ();
}

impl Handler<VecActor> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: VecActor, ctx: &mut Self::Context) -> Self::Result {
        ctx.binary(msg.0);
    }
}

//
// struct ByteActor(pub Bytes);
//
// impl actix::Message for ByteActor {
//     type Result = ();
// }

impl Actor for WsConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let vec_actor: Recipient<VecActor> = ctx.address().recipient();
        let close_actor: Recipient<CloseActor> = ctx.address().recipient();
        let hb_actor: Recipient<HbActor> = ctx.address().recipient();
        let addr = ctx.address();
        let id = self.id;
        let sessions = self.server_state.sessions.clone();
        let acc = self.acc.clone();

        let fut = async move {
            let acc = acc.read().await;

            if let Some(acc) = &*acc {
                let msg = ServerMsg::LoginFromTokenComplete {
                    user_id: acc.email.clone(),
                };
                let bytes = msg.as_bytes(0);
                let Ok(bytes) = bytes else {
                    println!("Failed to serialize server msg: {}", bytes.err().unwrap());
                    close_actor.do_send(CloseActor);
                    return;
                };
                vec_actor.do_send(VecActor(bytes));
            }
            let mut sessions = sessions.write().await;
            sessions.insert(id, addr);
            hb_actor.do_send(HbActor);
        };
        let fut = actix::fut::wrap_future::<_, Self>(fut);
        let _a = ctx.spawn(fut);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        ctx.stop();

        let sessions = self.server_state.sessions.clone();
        let id = self.id;

        spawn_local(async move {
            let mut sessions = sessions.write().await;
            sessions.remove(&id);
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, msg: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
                //ctx.binary(PING_MSG_BYTES);
            }
            Ok(ws::Message::Pong(msg)) => {
                self.hb = Instant::now();
                //   println!("PONG");
                //ctx.binary(PING_MSG_BYTES);
            }
            Ok(ws::Message::Text(_text)) => {}
            Ok(ws::Message::Binary(bytes)) => {
                //let ctx = Arc::new(ctx);
                let db = self.server_state.db.clone();
                let pepper = self.server_state.pepper.clone();
                let throttle_time = self.server_state.throttle_time.clone();
                let jwt_secret = self.server_state.jwt_secret.clone();
                let acc = self.acc.clone();
                let ip = self.ip.clone();
                let recipient: Recipient<VecActor> = ctx.address().recipient();
                let fut = async move {
                    let client_msg = ClientMsg::from_bytes(&bytes.to_vec());
                    let Ok((id, client_msg)) = client_msg else {
                        println!(
                            "Failed to convert bytes to client msg: {}",
                            client_msg.err().unwrap()
                        );
                        //let bytes = rkyv::to_bytes::<_, 256>(&ServerMsg::Reset);
                        let bytes = ServerMsg::Reset.as_bytes(0);
                        let Ok(bytes) = bytes else {
                            println!("Failed to serialize server msg: {}", bytes.err().unwrap());
                            return;
                        };
                        //ctx.binary(bytes.into_vec());
                        recipient.do_send(VecActor(bytes));
                        return;
                    };

                    let throttle = {
                        let mut throttle_time = throttle_time.write().await;
                        // let Ok(mut throttle_time) = throttle_time else {
                        //     println!("Failed to get throttle_time write lock.");
                        //     return;
                        // };
                        let throttle_time = &mut *throttle_time;
                        let path: WsPath = (&client_msg).into();
                        let current_time = Utc::now().timestamp_millis();
                        let duration = path.to_ms();
                        let count = path.to_count();

                        client_msg.throttle(throttle_time, &ip, path, current_time, duration, count)
                    };

                    if throttle {
                        println!("Connection for ip {} throttled.", ip);
                        return;
                    }

                    //println!("1");

                    let server_msg: Result<ServerMsg, ServerMsgCreationError> = match client_msg {
                        ClientMsg::GalleryInit { amount, from } => {
                            db.img_aggregate_gallery(amount, from)
                                .await
                                .or_else(|e| Err(ServerMsgCreationError::from(e)))
                            // MyWs::gallery_handler(db, amount, from).await
                        }
                        ClientMsg::UserGalleryInit {
                            amount,
                            from,
                            user_id,
                        } => db
                            .img_aggregate_user_gallery(amount, from, &user_id)
                            .await
                            .or_else(|e| Err(ServerMsgCreationError::from(e))),
                        ClientMsg::User { user_id } => db
                            .user_find_one(&user_id)
                            .await
                            .and_then(|user| Ok(ServerMsg::Profile(user)))
                            .or_else(|e| Err(ServerMsgCreationError::from(e))),
                        ClientMsg::Login { email, password } => {
                            ws_login(db, email, password, pepper, jwt_secret).await
                        }
                        ClientMsg::Register { email, password } => {
                            ws_register(db, pepper, email, password).await
                        }
                        ClientMsg::Logout => ws_logout(acc).await,
                    };
                    //println!("8");

                    let Ok(server_msg) = server_msg else {
                        println!("Failed to create server msg: {}", server_msg.err().unwrap());
                        return;
                    };

                    //println!("222222222 {:?}", server_msg);

                    let bytes = server_msg.as_bytes(id);
                    //let bytes = rkyv::to_bytes::<_, 256>(&server_msg);
                    let Ok(bytes) = bytes else {
                        println!("Failed to serialize server msg: {}", bytes.err().unwrap());
                        return;
                    };

                    //ctx.binary(bytes.into_vec());
                    recipient.do_send(VecActor(bytes));
                };
                let fut = actix::fut::wrap_future::<_, Self>(fut);
                let a = ctx.spawn(fut);
                //a.
            }
            Ok(ws::Message::Close(reason)) => ctx.close(reason),
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
            something => {
                println!("BOOOM {:?}", something);
            }
        }
    }
}

// jsonwebtoken::errors::Error
#[derive(Error, Debug)]
pub enum ServerMsgCreationError {
    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),

    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),

    #[error("RwLock error: {0}")]
    RwLock(String),
}
