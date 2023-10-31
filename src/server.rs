use cfg_if::cfg_if;

use rkyv::validation::validators::DefaultValidator;
use rkyv::validation::CheckTypeError;
use rkyv::with::ArchiveWith;
use rkyv::{Archive, Deserialize, Serialize};
use thiserror::Error;

// struct Test;
//
// impl ArchiveWith<bson::DateTime> for Test {
//     unsafe fn resolve_with(
//             field: &bson::DateTime,
//             pos: usize,
//             resolver: Self::Resolver,
//             out: *mut Self::Archived,
//         ) {
//         field
//         field.resolve(pos, (), out);
//     }
// }

#[derive(rkyv::Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub struct ServerMsgImg {
    pub user_id: String,
    pub msg_id: String,
    pub org_hash: String,
    pub format: String,
    pub has_high: bool,
    pub has_medium: bool,
    pub has_low: bool,
    pub modified_at: i64,
    pub created_at: i64,
}

#[derive(rkyv::Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub enum ServerMsg {
    Imgs(Vec<ServerMsgImg>),
}

#[derive(Error, Debug)]
pub enum WebSerializeError {
    #[error("Invalid bytes, error: {0}")]
    InvalidBytes(String),
}

impl ServerMsg {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WebSerializeError> {
        // let server_msg = rkyv::check_archived_root::<ServerMsg>(&bytes[..]);
        // let Ok(server_msg) = server_msg else {
        //     return Err(ServerMsgFromBytesError::InvalidBytes(format!(
        //         "Received invalid binary msg: {}",
        //         server_msg.err().unwrap()
        //     )));
        // };
        //
        // let server_msg: Result<ServerMsg, rkyv::Infallible> = server_msg.deserialize(&mut rkyv::Infallible);
        // let Ok(server_msg) = server_msg else {
        //     return Err(ServerMsgFromBytesError::InvalidBytes(format!(
        //         "Received invalid binary msg: {:?}",
        //         server_msg.err().unwrap()
        //     )));
        // };

        //let server_msg: ServerMsg = rkyv::check_archived_root::<ServerMsg>(&bytes[..]).unwrap().deserialize(&mut rkyv::Infallible).unwrap().into();

        let server_msg: Self = rkyv::check_archived_root::<Self>(bytes)
            .or_else(|e| {
                Err(WebSerializeError::InvalidBytes(format!(
                    "Received invalid binary msg: {}",
                    e
                )))
            })?
            .deserialize(&mut rkyv::Infallible)
            .or_else(|e| {
                Err(WebSerializeError::InvalidBytes(format!(
                    "Received invalid binary msg: {:?}",
                    e
                )))
            })?;

        Ok(server_msg)
    }
}

#[derive(rkyv::Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub enum ClientMsg {
    GalleryInit { amount: u8, from: Option<String> },
}

impl ClientMsg {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WebSerializeError> {
        // let server_msg = rkyv::check_archived_root::<ServerMsg>(&bytes[..]);
        // let Ok(server_msg) = server_msg else {
        //     return Err(ServerMsgFromBytesError::InvalidBytes(format!(
        //         "Received invalid binary msg: {}",
        //         server_msg.err().unwrap()
        //     )));
        // };
        //
        // let server_msg: Result<ServerMsg, rkyv::Infallible> = server_msg.deserialize(&mut rkyv::Infallible);
        // let Ok(server_msg) = server_msg else {
        //     return Err(ServerMsgFromBytesError::InvalidBytes(format!(
        //         "Received invalid binary msg: {:?}",
        //         server_msg.err().unwrap()
        //     )));
        // };

        //let server_msg: ServerMsg = rkyv::check_archived_root::<ServerMsg>(&bytes[..]).unwrap().deserialize(&mut rkyv::Infallible).unwrap().into();

        let server_msg: Self = rkyv::check_archived_root::<Self>(bytes)
            .or_else(|e| {
                Err(WebSerializeError::InvalidBytes(format!(
                    "check_archived_root failed: {}",
                    e
                )))
            })?
            .deserialize(&mut rkyv::Infallible)
            .or_else(|e| {
                Err(WebSerializeError::InvalidBytes(format!(
                    "deserialize failed: {:?}",
                    e
                )))
            })?;

        Ok(server_msg)
    }
}

cfg_if! {
if #[cfg(feature = "ssr")] {
use actix_web::web::Bytes;
use futures::TryStreamExt;
use mongodb::bson::{doc, Binary};
use std::collections::HashMap;
use actix::{Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler, Message, Recipient, StreamHandler, WrapFuture};
use actix_files::Files;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws::{self, CloseCode, CloseReason, ProtocolError};
use futures::{future, select, try_join, TryFutureExt};
use leptos::get_configuration;
use leptos_actix::{generate_route_list, LeptosRoutes};
use rand::Rng;

use async_std::task;
use dotenv::dotenv;
use futures::future::join_all;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandGroup, CommandResult, GroupOptions, StandardFramework};
use serenity::prelude::*;

use actix_web::dev::Server;
use std::env;
use std::sync::Arc;
use actix_web::web::BufMut;
use image::EncodableLayout;


struct MyWs {
    id: uuid::Uuid,
    server_state: ServerState
}

// struct Connect {
//     pub addr: Recipient<MSG>,
// }
//
// impl actix::Message for Connect {
//     type Result = ();
// }



impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("started what? {}", self.id);

        let sessions = self.server_state.sessions.try_lock();
        let Ok(mut sessions) = sessions else {
            let error = sessions.err().unwrap();
            println!("Locking WS sessions error: {}", error);
            ctx.close(Some(CloseReason::from(CloseCode::Error)));
            return;
        };

        //ctx.binary()

        sessions.insert(self.id, ctx.address());


        //
        //
        // //BROKEN
        // // ctx.address()
        // //     .send(MSG("NOOOOOOOOOOOOOOOOOO".to_string()))
        // //     .into_actor(self)
        // //     .then(|res, act, ctx| {
        // //         match res {
        // //             Ok(res) => {
        // //                 println!("how does this even make sense");
        // //                 ().start()
        // //             }
        // //             _ => {
        // //                 println!("started error???");
        // //                 ctx.stop()
        // //             }
        // //         }
        // //         println!("started READY???");
        // //         actix::fut::ready(())
        // //     })
        // //     .wait(ctx);
        //
        // let ad = ctx.address();
        //
        // //THIS ONE WORKS FINE:
        // ctx.address()
        //     .do_send(MSG("NOOOOOOOOOOOOOOOOOO".to_string()));
        //
        // println!("WHERES THE DUCK {}", self.id);
    }
}

struct VecMsg(pub Vec<u8>);

impl actix::Message for VecMsg {
    type Result = ();
    //type Result = actix::ResponseFuture<Result<(), ()>>;
}


struct MSG(pub Bytes);

impl actix::Message for MSG {
    type Result = ();
    //type Result = actix::ResponseFuture<Result<(), ()>>;
}

impl Handler<VecMsg> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: VecMsg, ctx: &mut Self::Context) -> Self::Result {
        ctx.binary(msg.0);
    }
}

impl Handler<MSG> for MyWs {
    // type Result = actix::ResponseFuture<Result<(), ()>>;
    //type Result = Result<(), ()>;
    type Result = ();

    fn handle(&mut self, msg: MSG, ctx: &mut Self::Context) -> () {
        let db = self.server_state.db.clone();
        let recipient = ctx.address().recipient();
        let client_msg = ClientMsg::from_bytes(&msg.0.to_vec());
        let Ok(client_msg) = client_msg else {
            println!("Failed to convert bytes to client msg: {}", client_msg.err().unwrap());
            return;
        };
        let fut = async move {
            let db = db;
            let find_options = mongodb::options::FindOptions::builder().sort(doc!{"created_at": 1}).build();
            let imgs = db.collection_img.find(doc!{}, Some(find_options)).await;
            let Ok(mut imgs) = imgs else {
                println!("Error fetching imgs: {}", imgs.err().unwrap());
                return;
            };

            let mut send_this: Vec<ServerMsgImg> = Vec::new();
            loop {
                let img = imgs.try_next().await;
                let Ok(Some(img)) = img else {
                    println!("last of img.");
                    break;
                };
                println!("IMG: {:#?}", img);
                let server_msg_img = ServerMsgImg {
                    msg_id: img.msg_id,
                    format: img.format,
                    user_id: img.user_id,
                    org_hash: img.org_hash,
                    has_low: img.has_low,
                    has_medium: img.has_medium,
                    has_high: img.has_high,
                    modified_at: img.modified_at.timestamp_millis(),
                    created_at: img.created_at.timestamp_millis(),
                };
                send_this.push(server_msg_img);
            }

            let msg = ServerMsg::Imgs(send_this);
            let bytes = rkyv::to_bytes::<_, 256>(&msg).unwrap();
           recipient.do_send(VecMsg(bytes.into_vec()));
        //         let recipient = ctx.address().recipient();
        //         let client_msg = ClientMsg::from_bytes(&bytes.to_vec());
        //         let Ok(client_msg) = client_msg else {
        //             println!("Failed to convert bytes to client msg: {}", client_msg.err().unwrap());
        //             return;
        //         };
           println!("IS THIS WORKING OR NOT");
        };
        let fut = actix::fut::wrap_future::<_, Self>(fut);
        let a = ctx.spawn(fut);
        a;
        //Ok(());
        // Box::pin(async move {
        //    // let client_msg = ClientMsg::from_bytes(&bytes.to_vec());
        //     // let Ok(client_msg) = client_msg else {
        //     //     println!("Failed to convert bytes to client msg: {}", client_msg.err().unwrap());
        //     //     return;
        //     // };
        //     println!("IM TRYING TO SEND THE MSG: {:?}", &msg.0);
        //     ctx.binary(msg.0);
        //     println!("HELLO");
        //     Ok(())
        // }).intok
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    //type Result = actix::ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        // Box::pin(async move {
        //     println!("HELLO");
        //
        // })

        match msg {
            Ok(ws::Message::Ping(msg)) => {
                println!("BING BING");
                ctx.pong(&msg)
            }
            Ok(ws::Message::Text(text)) => {
                println!("TEXT RECEIVED {}", text);
            }
            Ok(ws::Message::Binary(bytes)) => {
                ctx.address().do_send(MSG(bytes));
                println!("wow");

            },
            Ok(ws::Message::Close(reason)) => {
                println!("WTF HAPPENED {:?}", reason);
                ctx.close(reason)
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
            _ => {
                println!("BOOOM");
            }
        }
        // match msg {
        //     Ok(ws::Message::Ping(msg)) => {
        //         println!("BING BING");
        //         ctx.pong(&msg)
        //     }
        //     Ok(ws::Message::Text(text)) => {
        //
        //         println!("TEXT RECEIVED {}", text);
        //         //let a = ctx.address();
        //         //a.do_send(MSG("wow".to_string()));
        //         // let msg = ServerMsg::Str("yo bro".to_string());
        //         // let bytes = rkyv::to_bytes::<_, 256>(&msg).unwrap();
        //         // ctx.binary(bytes.into_vec());
        //
        //
        //
        //         //ctx.text(text)
        //     }
        //     Ok(ws::Message::Binary(bytes)) => {
        //         // let fut = async move {
        //         //     let imgs = self.server_state.db.collection_img.find(doc!{}, None).await;
        //         //     let Ok(imgs) = imgs else {
        //         //         println!("Error fetching imgs: {}", imgs.err().unwrap());
        //         //         return;
        //         //     };
        //         //     println!("Imgs fetched: {:?}", imgs);
        //         // };
        //         //
        //         // //fut.into_actor(self).spawn(ctx);
        //         // //let fut = Arc::new(fut);
        //         // let fut = actix::fut::wrap_future::<_, Self>(fut);
        //         // ctx.spawn(fut);
        //         // //ctx.add
        //
        //         let db = self.server_state.db.clone();
        //         let recipient = ctx.address().recipient();
        //         let client_msg = ClientMsg::from_bytes(&bytes.to_vec());
        //         let Ok(client_msg) = client_msg else {
        //             println!("Failed to convert bytes to client msg: {}", client_msg.err().unwrap());
        //             return;
        //         };
        //         //let msg = msg.clone();
        //         let fut = async move {
        //             match client_msg {
        //                     ClientMsg::GalleryInit { amount, from } => {
        //                         println!("Received: {} {:?}", amount, from);
        //
        //                         let find_options = mongodb::options::FindOptions::builder().sort(doc!{"created_at": 1}).build();
        //                         let imgs = db.collection_img.find(doc!{}, Some(find_options)).await;
        //                         // let imgs = self.server_state.db.collection_img.find(doc!{}, None).await;
        //                         let Ok(mut imgs) = imgs else {
        //                             println!("Error fetching imgs: {}", imgs.err().unwrap());
        //                             return;
        //                         };
        //
        //                         let mut send_this: Vec<ServerMsgImg> = Vec::new();
        //                         loop {
        //                             let img = imgs.try_next().await;
        //                             let Ok(Some(img)) = img else {
        //                                 println!("last of img.");
        //                                 break;
        //                             };
        //                             println!("IMG: {:#?}", img);
        //                             let server_msg_img = ServerMsgImg {
        //                                 msg_id: img.msg_id,
        //                                 format: img.format,
        //                                 user_id: img.user_id,
        //                                 org_hash: img.org_hash,
        //                                 has_low: img.has_low,
        //                                 has_medium: img.has_medium,
        //                                 has_high: img.has_high,
        //                                 modified_at: img.modified_at.timestamp_millis(),
        //                                 created_at: img.created_at.timestamp_millis(),
        //                             };
        //                             send_this.push(server_msg_img);
        //                         }
        //                         let bytes = bytes.to_vec();
        //                         println!("RECEIVED BYTES: {:?}", &bytes);
        //
        //                         let client_msg = ClientMsg::from_bytes(&bytes);
        //                         let Ok(client_msg) = client_msg else {
        //                             println!("Decoding client msg error: {}", client_msg.err().unwrap());
        //                             return;
        //                         };
        //
        //                         let msg = ServerMsg::Imgs(send_this);
        //                         let bytes = rkyv::to_bytes::<_, 256>(&msg).unwrap();
        //                         recipient.send(MSG(bytes.into_vec()));
        //
        //                     }
        //             };
        //             //println!("Imgs fetched: {:#?}", imgs);
        //
        //         };
        //
        //         // fut.into_actor(self).spawn(ctx);
        //         let fut = actix::fut::wrap_future::<_, Self>(fut);
        //         ctx.spawn(fut);
        //         println!("wow");
        //
        //
        //           // let client_msg = rkyv::check_archived_root::<ClientMsg>(&bytes[..]);
        //           // let Ok(client_msg) = client_msg else {
        //           //       println!("Received invalid binary msg: {}", client_msg.err().unwrap());
        //           //       return;
        //           // };
        //           //
        //           // let client_msg: Result<ClientMsg, rkyv::Infallible> = client_msg.deserialize(&mut rkyv::Infallible);
        //           // let Ok(client_msg) = client_msg else {
        //           //       println!("Received invalid binary msg: {:?}", client_msg.err().unwrap());
        //           //       return;
        //           // };
        //
        //     },
        //     Ok(ws::Message::Close(reason)) => {
        //         println!("WTF HAPPENED {:?}", reason);
        //         ctx.close(reason)
        //     }
        //     Err(e) => {
        //         println!("ERROR: {:?}", e);
        //     }
        //     _ => {
        //         println!("BOOOM");
        //     }
        // }
        // //let fut = Arc::new(fut);
        // //let fut = actix::fut::wrap_future::<_, Self>(fut);
        // //ctx.spawn(fut);
        //
        // //ctx.add

    }
}

impl Actor for ServerState {
    type Context = actix::Context<Self>;
}

async fn index(
    req: HttpRequest,
    stream: web::Payload,
    //srv: web::Data<Addr<MyWs>>,
    //srv: web::Data<Addr<ServerState>>
    server_state: actix_web::web::Data<ServerState>
) -> Result<HttpResponse, Error> {

    let resp = ws::start(
        MyWs {
            id: uuid::Uuid::new_v4(),
            server_state: server_state.get_ref().to_owned().clone()
        },
        &req,
        stream,
    );
    //resp.unwrap().await.unwrap().
    let ip = req.peer_addr().unwrap().ip();
    let port = req.peer_addr().unwrap().port();

    println!("{:?}:{} {:?}", ip, port, resp);

    resp
}

//#[actix_web::get("favicon.ico")]
pub async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open(format!(
        "assets/favicon.ico"
    ))?)
}

#[derive(Clone)]
pub struct ServerState {
    sessions: Arc<Mutex<HashMap<uuid::Uuid,Addr<MyWs>>>>,
    db: crate::database::DB
}

pub async fn create_server(db: crate::database::DB) -> Server {
    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    let routes = generate_route_list(crate::app::App);
    println!("listening on http://{}", &addr);

    // let db_mutex = Mutex::new(db);
    //
    // let db_mutex_arc = Arc::new(db_mutex);



    // let server_state = web::Data::new(ServerState {
    //     db: db_mutex_arc
    // });
    // let server_state = ServerState {
    //     db: std::sync::Mutex::new(db)
    // };
    //let server_state_arc = Arc::new(server_state);
    //let server_state_arc_mutex = Mutex::new(server_state_arc);

    let sessions = Arc::new(Mutex::new(HashMap::<uuid::Uuid, Addr<MyWs>>::new()));


    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            //.service(favicon)
            .app_data(web::Data::new(ServerState {
                sessions: sessions.clone(),
                db: db.clone()
            }))
            .route("/favicon.ico", web::get().to(favicon))
            .route("/ws/", web::get().to(index))
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", site_root))
            // serve the favicon from /favicon.ico

            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                crate::app::App,
            )
            //.app_data(web::Data::new(leptos_options.to_owned()))
        //.wrap(middleware::Compress::default())
    })
    .workers(2)
    .bind(&addr)
    .unwrap()
    .run()
}
}
}
