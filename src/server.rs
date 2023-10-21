use actix::{
    Actor, ActorContext, ActorFutureExt, AsyncContext, ContextFutureSpawner, Handler, Message,
    Recipient, StreamHandler, WrapFuture,
};
use actix_files::Files;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws::{self, ProtocolError};
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
use wasm_bindgen::__rt::Start;

struct MyWs {
    id: u32,
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

        //BROKEN
        // ctx.address()
        //     .send(MSG("NOOOOOOOOOOOOOOOOOO".to_string()))
        //     .into_actor(self)
        //     .then(|res, act, ctx| {
        //         match res {
        //             Ok(res) => {
        //                 println!("how does this even make sense");
        //                 ().start()
        //             }
        //             _ => {
        //                 println!("started error???");
        //                 ctx.stop()
        //             }
        //         }
        //         println!("started READY???");
        //         actix::fut::ready(())
        //     })
        //     .wait(ctx);

        //THIS ONE WORKS FINE:
        ctx.address()
            .do_send(MSG("NOOOOOOOOOOOOOOOOOO".to_string()));

        println!("WHERES THE DUCK {}", self.id);
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
                println!("WHATS UP DUCK {}", text);
                let a = ctx.address();
                a.do_send(MSG("wow".to_string()));

                ctx.text(text)
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

async fn index(
    req: HttpRequest,
    stream: web::Payload,
    //srv: web::Data<Addr<MyWs>>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        MyWs {
            id: rand::thread_rng().gen_range(500..1000),
        },
        &req,
        stream,
    );
    let ip = req.peer_addr().unwrap().ip();
    let port = req.peer_addr().unwrap().port();

    println!("{:?}:{} {:?}", ip, port, resp);

    resp
}

#[actix_web::get("favicon.ico")]
pub async fn favicon(
    leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}

pub async fn create_server() -> Server {
    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    let routes = generate_route_list(crate::app::App);
    println!("listening on http://{}", &addr);

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .route("/ws/", web::get().to(index))
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", site_root))
            // serve the favicon from /favicon.ico
            .service(favicon)
            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                crate::app::App,
            )
            .app_data(web::Data::new(leptos_options.to_owned()))
        //.wrap(middleware::Compress::default())
    })
    .workers(2)
    .bind(&addr)
    .unwrap()
    .run()
}
