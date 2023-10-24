use cfg_if::cfg_if;

use rkyv::{Archive, Deserialize, Serialize};
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub enum ServerMsg {
    Str(String)
}

cfg_if! {
if #[cfg(feature = "ssr")] {

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

struct Connect {
    pub addr: Recipient<MSG>,
}

impl actix::Message for Connect {
    type Result = ();
}



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

struct MSG(pub String);

impl actix::Message for MSG {
    type Result = ();
}

impl Handler<MSG> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: MSG, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                println!("BING BING");
                ctx.pong(&msg)
            }
            Ok(ws::Message::Text(text)) => {
                println!("TEXT RECEIVED {}", text);
                //let a = ctx.address();
                //a.do_send(MSG("wow".to_string()));
                let msg = ServerMsg::Str("yo bro".to_string());
                let bytes = rkyv::to_bytes::<_, 256>(&msg).unwrap();
                ctx.binary(bytes.into_vec());

                //ctx.text(text)
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
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



