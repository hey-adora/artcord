use crate::database::models::acc::Acc;
use crate::server::client_msg::{ClientMsg, WsPath};
use crate::server::create_server::ServerState;
use crate::server::registration_invalid::{RegistrationInvalidMsg, BCRYPT_COST};
use crate::server::server_msg::ServerMsg;
use actix::{Actor, Addr, AsyncContext, Handler, Recipient, StreamHandler};
use actix_web::web::Bytes;
use actix_web_actors::ws::{self, CloseCode, CloseReason, ProtocolError};
use chrono::Utc;
use rand::Rng;
use std::net::{IpAddr, SocketAddr};
use std::sync::LockResult;
use thiserror::Error;

pub struct WsConnection {
    pub id: uuid::Uuid,
    pub ip: IpAddr,
    pub server_state: ServerState,
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
        let pepper = self.server_state.pepper.clone();
        let throttle_time = self.server_state.throttle_time.clone();
        let ip = self.ip.clone();
        let recipient: Recipient<_> = ctx.address().recipient();
        let fut = async move {
            let client_msg = ClientMsg::from_bytes(&msg.0.to_vec());
            let Ok(client_msg) = client_msg else {
                println!(
                    "Failed to convert bytes to client msg: {}",
                    client_msg.err().unwrap()
                );
                let bytes = rkyv::to_bytes::<_, 256>(&ServerMsg::Reset);
                let Ok(bytes) = bytes else {
                    println!("Failed to serialize server msg: {}", bytes.err().unwrap());
                    return;
                };
                recipient.do_send(VecActor(bytes.into_vec()));
                return;
            };

            let throttle = {
                let throttle_time = throttle_time.write();
                let Ok(mut throttle_time) = throttle_time else {
                    println!("Failed to get throttle_time write lock.");
                    return;
                };
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
                    println!("LOGIN '{}' '{}'", email, password);

                    Ok(ServerMsg::None)
                }
                ClientMsg::Register { email, password } => {
                    // let salt: String = (0..256)
                    //     .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
                    //     .collect();
                    let email_code: String = (0..25)
                        .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
                        .collect();
                    let (invalid, email_error, password_error) =
                        RegistrationInvalidMsg::validate_registration(&email, &password);
                    if invalid == false {
                        let password = format!("{}{}", &password, &pepper);
                        let password_hash = bcrypt::hash(&password, BCRYPT_COST);
                        if let Ok(password_hash) = password_hash {
                            //let verified = bcrypt::verify(&password, &password_hash);
                            // if let Ok(verified) = verified {
                            //     println!(
                            //         "REGISTER:\nemail:'{}'\npassword:'{}'\npassword_hash:'{}'\npassword_verified:'{}'",
                            //         email, password, password_hash, verified
                            //     );
                            //
                            //     Ok(ServerMsg::None)
                            // } else {
                            //     Err(ServerMsgCreationError::from(verified.err().unwrap()))
                            // }
                            // println!(
                            //     "REGISTER:\nemail:'{}'\npassword:'{}'\npassword_hash:'{}'\npassword_verified:'{}'",
                            //     email, password, password_hash, verified
                            // );
                            //db.
                            let acc = Acc::new(&email, &password_hash, &email_code);
                            let result = db
                                .create_acc(acc)
                                .await
                                .and_then(|e| Ok(ServerMsg::RegistrationCompleted))
                                .or_else(|e| Err(ServerMsgCreationError::from(e)));

                            result
                        } else {
                            Err(ServerMsgCreationError::from(password_hash.err().unwrap()))
                        }
                    } else {
                        println!(
                            "INVALID: {} {:?} {:?}",
                            invalid, email_error, password_error
                        );
                        Ok(ServerMsg::None)
                    }

                    // let Ok(password_hash) = password_hash else {
                    //     return ServerMsgCreationError::from(password_hash.err().unwrap());
                    // };
                }
            };

            let Ok(server_msg) = server_msg else {
                println!("Failed to create server msg: {}", server_msg.err().unwrap());
                return;
            };

            println!("222222222 {:?}", server_msg);

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
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(_text)) => {}
            Ok(ws::Message::Binary(bytes)) => {
                ctx.address().do_send(ByteActor(bytes));
            }
            Ok(ws::Message::Close(reason)) => ctx.close(reason),
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
            _ => {
                println!("BOOOM");
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ServerMsgCreationError {
    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),

    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
}
