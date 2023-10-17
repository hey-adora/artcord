use actix_web::web::Json;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};

use actix::{
    Actor, ActorContext, ActorFuture, ActorFutureExt, Addr, AsyncContext, Context,
    ContextFutureSpawner, Handler, Message, Recipient, Running, StreamHandler, WrapFuture,
};

use actix_web_actors::ws::{self, ProtocolError};
use futures::future;
use rand::Rng;
use uuid::Uuid;
use wasm_bindgen::__rt::Start;

// struct Session {
//     //pub addr: Addr<MyWs>,
// }

// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct WsMessage(pub String);
//
// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct Connect {
//     pub addr: Recipient<WsMessage>,
// }
//
// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct Disconnect {
//     pub wtf: String,
// }
//
// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct ClientActorMessage {
//     pub msg: String,
// }
//
//
// struct Server {
//     me: Recipient<WsMessage>
// }
//
// impl Actor for Server {
//     type Context = Context<Self>;
// }

// impl Handler<Message> for Server {
//     type Result = ();
//
//     fn handle(&mut self, msg: Message, _: &mut Context<Self>) {
//         println!("SEND WHAT???");
//         //self.send_message(&msg.room, msg.msg.as_str(), msg.id);
//     }
// }

struct MyWs {
    //pub addr: Addr<MyWs>,
    id: u32,
}

// impl Handler<WsMessage> for MyWs {
//     type Result = ();
//
//     fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
//         ctx.text(msg.0);
//     }
// }

// impl Handler<Connect> for Server {
//
//     type Result = ();
//     fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
//         let s = &self.me;
//         s.
//     }
// }

struct Connect {
    pub addr: Recipient<MSG>,
}

impl actix::Message for Connect {
    type Result = ();
}

impl Handler<Connect> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) {
        println!("Someone joined??FE?FE?EF?");

        // notify all users in same room
        //self.se("main", "Someone joined", 0);

        // register session with random id
        //let id = self.rng.gen::<usize>();
        //self.sessions.insert(id, msg.addr);

        // auto join session to main room
        //self.rooms.get_mut("main").unwrap().insert(id);

        // send id back
        //id
    }
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("started what? {}", self.id);

        let addr = ctx.address();
        addr.do_send(MSG("HISSSSSSSSSSSSSS".to_string()));
        addr.do_send(Connect {
            addr: addr.clone().recipient(),
        });

        // let b = addr
        //     .send(MSG("NOOOOOOOOOOOOOOOOOO".to_string()))
        //     .into_actor(self)
        //     .then(|res, act, ctx| {
        //         match res {
        //             Ok(res) => {
        //                 println!("how does this even make sense");
        //
        //                 ().start()
        //             }
        //             // something is wrong with chat server
        //             _ => {
        //                 println!("started error???");
        //                 ctx.stop()
        //             }
        //         }
        //         println!("started READY???");
        //         actix::fut::ready(())
        //     });

        // futures::executor::block_on(async {
        //     addr.send(MSG("NOOOOOOOOOOOOOOOOOO".to_string()))
        //         .await
        //         .unwrap()
        // });
        //b.wait(ctx);

        // addr.send(Connect {
        //     addr: addr.clone().recipient(),
        // })
        // .into_actor(self)
        // .then(|res, act, ctx| {
        //     match res {
        //         Ok(res) => {
        //             println!("how does this even make sense");
        //
        //             ().start()
        //         }
        //         // something is wrong with chat server
        //         _ => {
        //             println!("started error???");
        //             ctx.stop()
        //         }
        //     }
        //     println!("started READY???");
        //     actix::fut::ready(())
        // });

        // addr.send(Connect {
        //     addr: addr.clone().recipient(),
        // })
        // .into_actor(self)
        // .then(|res, act, ctx| {
        //     match res {
        //         Ok(res) => {
        //             println!("how does this even make sense");
        //
        //             ().start()
        //         }
        //         // something is wrong with chat server
        //         _ => {
        //             println!("started error???");
        //             ctx.stop()
        //         }
        //     }
        //     println!("started READY???");
        //     actix::fut::ready(())
        // })
        // .wait(ctx);

        // ctx.address()
        //     .send(MSG("soooooooo, wyd?".to_string()))
        //     .into_actor(self)
        //     .then(|res, _, ctx| {
        //         match res {
        //             Ok(_res) => {
        //                 println!("how does this even make sense");
        //
        //                 ().start()
        //             }
        //             _ => {
        //                 println!("started error???");
        //                 ctx.stop()
        //             }
        //         }
        //         actix::fut::ready(())
        //     })
        //     .wait(ctx);
    }

    // fn started(&mut self, ctx: &mut Self::Context) {
    //     let addr = ctx.address();
    //     self.addr
    //         .send(Connect {
    //             addr: addr.recipient(),
    //         })
    //         .into_actor(self)
    //         .then(|res, _, ctx| match res {
    //             Ok(_res) => ().start(),
    //             _ => ctx.stop(),
    //         })
    //         .wait();
    // }
    //
    // fn stopping(&mut self, _: &mut Self::Context) -> Running {
    //     self.addr.do_send(Disconnect {
    //         wtf: String::from("WOWWOWOWOWOWOOWOWOWOWOW"),
    //     });
    //     Running::Stop
    // }
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
                // a.send(MSG("wow".to_string())).into_actor(self).then(|res, _, ctx| {
                //     match res {
                //         Ok(_res) => (),
                //         _ => ctx.stop()
                //     }
                //     actix::fut::ready(())
                // }).wait(ctx);

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

// #[derive(actix::Message)]
// #[rtype(result = "()")]
// pub struct Message {
//     pub msg: String,
// }

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
    // let _ = srv.send(Message {
    //     msg: String::from("wwwwww"),
    // });

    resp
}

// #[get("/{group_id}")]
// pub async fn start_connection(
//     req: HttpRequest,
//     stream: Payload,
//     srv: web::Data<Addr<Lobby>>,
// ) -> Result<HttpResponse, Error> {
//     let ws = WsConn::new(group_id, srv.get_ref().clone());
//
//     let resp = ws::start(ws, &req, stream)?;
//     Ok(resp)
// }

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use artcord::app::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    println!("listening on http://{}", &addr);

    let web_server = HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .route("/ws/", web::get().to(index))
            //.service(wc_connection)
            //.service(web::resource("/ws/").to(index))
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", site_root))
            // serve the favicon from /favicon.ico
            .service(favicon)
            .leptos_routes(leptos_options.to_owned(), routes.to_owned(), App)
            .app_data(web::Data::new(leptos_options.to_owned()))
        //.wrap(middleware::Compress::default())
    })
    .workers(2)
    .bind(&addr)
    .unwrap()
    .run();

    let futs = [web_server];

    future::join_all(futs).await;

    Ok(())
}

// #[cfg(feature = "ssr")]
// #[get("/ws/")]
// pub async fn wc_connection(
//     req: HttpRequest,
//     stream: web::Payload,
//     //srv: web::Data<Addr<Server>>,
// ) -> Result<HttpResponse, Error> {
//     //let ws = WsConn::new(group_id, srv.get_ref().clone());
//     let ws = MyWs {};
//
//     let resp = ws::start(ws, &req, stream)?;
//     Ok(resp)
// }

#[cfg(feature = "ssr")]
#[actix_web::get("favicon.ico")]
async fn favicon(
    leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use artcord::app::*;
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(move || {
        // note: for testing it may be preferrable to replace this with a
        // more specific component, although leptos_router should still work
        view! { <App/> }
    });
}
