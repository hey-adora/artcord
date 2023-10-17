use actix::{
    Actor, ActorContext, ActorFutureExt, AsyncContext, ContextFutureSpawner, Handler, Message,
    Recipient, StreamHandler, WrapFuture,
};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws::{self, ProtocolError};
use futures::future;
use rand::Rng;
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
        ctx.address()
            .send(MSG("NOOOOOOOOOOOOOOOOOO".to_string()))
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        println!("how does this even make sense");
                        ().start()
                    }
                    _ => {
                        println!("started error???");
                        ctx.stop()
                    }
                }
                println!("started READY???");
                actix::fut::ready(())
            })
            .wait(ctx);

        // THIS ONE WORKS FINE:
        // ctx.address()
        //     .do_send(MSG("NOOOOOOOOOOOOOOOOOO".to_string()));

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
