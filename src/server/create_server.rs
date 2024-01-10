use actix_web::web::Bytes;
use futures::TryStreamExt;
use mongodb::bson::{doc};
use std::collections::HashMap;
use actix::{Actor, Addr, AsyncContext, Handler, Recipient, StreamHandler};
use actix_files::Files;
use actix_web::{web, App, Responder, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws::{self, CloseCode, CloseReason, ProtocolError};
use leptos::get_configuration;
use leptos_actix::{generate_route_list, LeptosRoutes};
use serenity::prelude::*;
use actix_web::dev::Server;
use std::{num::ParseIntError, sync::Arc};
use std::net::{IpAddr, SocketAddr};
use std::sync::RwLock;
use thiserror::Error;
use crate::server::client_msg::WsPath;
use crate::server::ws_connection::{WsConnection};

impl Actor for ServerState {
    type Context = actix::Context<Self>;
}

async fn index(
    req: HttpRequest,
    stream: web::Payload,
    server_state: actix_web::web::Data<ServerState>
) -> Result<HttpResponse, actix_web::Error> {
    let Some(peer) = req.peer_addr() else {
        println!("Error: failed to get peer_addr().");
        return HttpResponse::BadRequest().await;
    };

    ws::start(
        WsConnection {
            id: uuid::Uuid::new_v4(),
            ip: peer.ip(),
            server_state: server_state.get_ref().to_owned().clone()
        },
        &req,
        stream,
    )
}

pub async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("target/site/favicon.ico")?)
}

#[derive(Clone)]
pub struct ServerState {
    pub throttle_time: Arc<RwLock<HashMap<WsPath, (u64, HashMap<IpAddr, u64 >)>>>,
    pub sessions: Arc<RwLock<HashMap<uuid::Uuid, Addr<WsConnection > >>>,
    pub gallery_root_dir: Arc<str>,
    pub db: Arc<crate::database::DB>,
}



async fn overview(
    _req: HttpRequest,
    _stream: web::Payload,
    server_state: actix_web::web::Data<ServerState>
) -> impl Responder {
    let sessions = server_state.sessions.read();
    let Ok(sessions) = sessions else {
        let error = sessions.err().unwrap();
        return HttpResponse::InternalServerError().body(format!("Error: {}", error));
    };
    HttpResponse::Ok().body(format!("Live connection: {}", sessions.len()))
}

pub async fn create_server(db: Arc<crate::database::DB>, galley_root_dir: &str, assets_root_dir: &str) -> Server {
    let conf = get_configuration(None).await.unwrap();
    println!("CONFIG: {:#?}", &conf);
    let addr = conf.leptos_options.site_addr;
    let routes = generate_route_list(crate::app::App);
    println!("listening on http://{}", &addr);

    let sessions = Arc::new(RwLock::new(HashMap::<uuid::Uuid, Addr<WsConnection >>::new()));


    let galley_root_dir = galley_root_dir.to_string();
    let assets_root_dir = assets_root_dir.to_string();
    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        // let site_root = &leptos_options.site_root;
        println!("site root: {}", assets_root_dir.as_str());
        let pkg_url = format!("{}/pkg", assets_root_dir.as_str());
        println!("pkg dir: {}", pkg_url);

        App::new()
            .app_data(web::Data::new(ServerState {
                throttle_time:  Arc::new(RwLock::new(HashMap::new())),
                sessions: sessions.clone(),
                gallery_root_dir: Arc::from(galley_root_dir.as_str()),
                db: db.clone(),
            }))
            .route("/overview", web::get().to(overview))
            .route("/favicon.ico", web::get().to(favicon))
            .route("/ws/", web::get().to(index))
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .service(Files::new("/assets/gallery", galley_root_dir.clone()))
            .service(Files::new("/assets", assets_root_dir.as_str()))
            .service(Files::new("/pkg", pkg_url))

            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                crate::app::App,
            )
    })
        .workers(2)
        .bind(&addr)
        .unwrap()
        .run()
}

#[derive(Error, Debug)]
pub enum ServerMsgError {
    #[error("Casting error: {0}.")]
    Cast(#[from] ParseIntError),

    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Bson: {0}.")]
    Bson(#[from] mongodb::bson::de::Error),
}