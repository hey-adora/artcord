use actix_files::Files;

use actix_web::dev::Server;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};




use futures::StreamExt;
use leptos::logging::warn;
use leptos_actix::{generate_route_list, LeptosRoutes};

use tracing::{error, info, trace};
use std::fs::read_to_string;

use cfg_if::cfg_if;

use std::sync::Arc;

pub const TOKEN_SIZE: usize = 257;

// impl Actor for ServerState {
//     type Context = actix::Context<Self>;
// }

// async fn ws_route(
//     req: HttpRequest,
//     stream: web::Payload,
//     server_state: actix_web::web::Data<ServerState>,
// ) -> Result<HttpResponse, actix_web::Error> {
//     let Some(peer) = req.peer_addr() else {
//         println!("Error: failed to get peer_addr().");
//         return HttpResponse::BadRequest().await;
//     };

//     let cookie = req.cookie("token");
//     let db = server_state.db.clone();
//     // if let Some(cookie) = cookie {
//     //     println!("COOKIE {}", cookie.value());
//     // };
//     let acc: Option<Acc> = {
//         let acc_session = if let Some(cookie) = cookie {
//             let acc = db.acc_session_find_one(cookie.value()).await;
//             if let Ok(acc) = acc {
//                 acc
//             } else {
//                 None
//             }
//         } else {
//             None
//         };

//         if let Some(acc_session) = acc_session {
//             let acc = db.acc_find_one_by_id(&acc_session.acc_id).await;
//             if let Ok(acc) = acc {
//                 acc
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     };
//     //println!("ACC {:#?}", acc);

//     let a = ws::start(
//         WsConnection {
//             id: uuid::Uuid::new_v4(),
//             ip: peer.ip(),
//             acc: Arc::new(RwLock::new(acc)),
//             server_state: server_state.get_ref().to_owned().clone(),
//             hb: Instant::now(),
//         },
//         &req,
//         stream,
//     );

//     a
// }

// async fn login_token_route(
//     req: HttpRequest,
//     mut stream: web::Payload,
//     server_state: actix_web::web::Data<ServerState>,
// ) -> impl Responder {
//     let body = stream.next().await;
//     let Some(body) = body else {
//         return HttpResponse::BadRequest().body("Must contain token in body.");
//     };

//     let Ok(body) = body else {
//         return HttpResponse::BadRequest().body("Failed to get token from body.");
//     };

//     if body.len() == TOKEN_SIZE - 1 {
//         return HttpResponse::BadRequest()
//             .body(format!("Token is too long, it must be {}.", TOKEN_SIZE));
//     }

//     let token = String::from_utf8(body.to_vec());
//     let Ok(token) = token else {
//         return HttpResponse::BadRequest().body("Token must be in UTF8 standard.");
//     };

//     //println!("{:#?}", body);
//     let cookie = Cookie::build("token", token)
//         .domain("localhost")
//         .path("/ws")
//         .http_only(true)
//         .same_site(SameSite::Strict)
//         .secure(true)
//         .finish();

//     HttpResponse::Ok().cookie(cookie).finish()
// }

// async fn login_delete_token_route(
//     req: HttpRequest,
//     mut stream: web::Payload,
//     server_state: actix_web::web::Data<ServerState>,
// ) -> impl Responder {
//     let time = OffsetDateTime::from_unix_timestamp(0);
//     let Ok(time) = time else {
//         return HttpResponse::InternalServerError().body("Failed to create time.");
//     };

//     let cookie = Cookie::build("token", "deleted")
//         .domain("localhost")
//         .expires(time)
//         .path("/ws")
//         .http_only(true)
//         .same_site(SameSite::Strict)
//         .secure(true)
//         .finish();

//     HttpResponse::Ok().cookie(cookie).finish()
// }

// pub async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
//     Ok(actix_files::NamedFile::open("target/site/favicon.ico")?)
// }

// // #[derive(Clone)]
// // pub struct ServerState {
// //     pub throttle_time: Arc<RwLock<HashMap<WsPath, (u64, HashMap<IpAddr, u64>)>>>,
// //     pub sessions: Arc<RwLock<HashMap<uuid::Uuid, Addr<WsConnection>>>>,
// //     pub gallery_root_dir: Arc<String>,
// //     pub db: Arc<DB>,
// //     pub pepper: Arc<String>,
// //     pub jwt_secret: Arc<Vec<u8>>,
// // }

// async fn overview(
//     _req: HttpRequest,
//     _stream: web::Payload,
//     server_state: actix_web::web::Data<ServerState>,
// ) -> impl Responder {
//     let sessions = server_state.sessions.read().await;
//     HttpResponse::Ok().body(format!("Live connection: {}", sessions.len()))
// }

// pub async fn create_server(
//     db: Arc<DB>,
//     galley_root_dir: Arc<String>,
//     assets_root_dir: Arc<String>,
//     pepper: Arc<String>,
//     jwt_secret: Arc<Vec<u8>>,
// ) -> Server {
//     let conf = get_configuration(None).await.unwrap();
//     // let conf: ConfFile = ConfFile {
//     //     leptos_options: LeptosOptions {
//     //         output_name: "leptos_start5".to_string(),
//     //         site_root: "target/site".to_string(),
//     //         site_pkg_dir: "pkg".to_string(),
//     //         env: DEV,
//     //         site_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3000)),
//     //         reload_port: 3001,
//     //         reload_external_port: None,
//     //         reload_ws_protocol: WS,
//     //         not_found_path: "/404".to_string(),
//     //     },
//     // };
//     println!("CONFIG: {:#?}", &conf);
//     let addr = conf.leptos_options.site_addr;
//     let routes = generate_route_list(crate::app::App);
//     println!("listening on http://{}", &addr);
//     let sessions = Arc::new(RwLock::new(HashMap::<uuid::Uuid, Addr<WsConnection>>::new()));

//     //let galley_root_dir = galley_root_dir.to_string();
//     //let assets_root_dir = assets_root_dir.to_string();
//     HttpServer::new(move || {
//         let leptos_options = &conf.leptos_options;
//         // let site_root = &leptos_options.site_root;
//         println!("site root: {}", &*assets_root_dir);
//         let pkg_url = format!("{}/pkg", &*assets_root_dir);
//         println!("pkg dir: {}", pkg_url);

//         App::new()
//             .app_data(web::Data::new(ServerState {
//                 throttle_time: Arc::new(RwLock::new(HashMap::new())),
//                 sessions: sessions.clone(),
//                 gallery_root_dir: galley_root_dir.clone(),
//                 db: db.clone(),
//                 pepper: pepper.clone(),
//                 jwt_secret: jwt_secret.clone(),
//             }))
//             .route("/overview", web::get().to(overview))
//             .route("/login_token", web::post().to(login_token_route))
//             .route(
//                 "/login_delete_token",
//                 web::post().to(login_delete_token_route),
//             )
//             .route("/favicon.ico", web::get().to(favicon))
//             .route("/ws/", web::get().to(ws_route))
//             .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
//             .service(Files::new("/assets/gallery", &*galley_root_dir))
//             .service(Files::new("/assets", &*assets_root_dir))
//             .service(Files::new("/pkg", pkg_url))
//             .leptos_routes(
//                 leptos_options.to_owned(),
//                 routes.to_owned(),
//                 crate::app::App,
//             )
//     })
//     .workers(1)
//     .bind(&addr)
//     .unwrap()
//     .run()
// }

// #[derive(Error, Debug)]
// pub enum ServerMsgError {
//     #[error("Casting error: {0}.")]
//     Cast(#[from] ParseIntError),

//     #[error("Mongodb: {0}.")]
//     Mongo(#[from] mongodb::error::Error),

//     #[error("Bson: {0}.")]
//     Bson(#[from] mongodb::bson::de::Error),
// }

// #[cfg(test)]
// mod ClientMsgTests {
//     use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
//     use serde::{Deserialize, Serialize};

//     #[derive(Debug, Serialize, Deserialize)]
//     struct Acc {
//         name: String,
//         exp: u64,
//     }

//     #[test]
//     fn jwt() {
//         let key = b"secret";
//         let acc = Acc {
//             name: "wowza".to_string(),
//             exp: 0,
//         };
//         let header = Header::new(Algorithm::HS512);
//         let token =
//             encode(&header, &acc, &EncodingKey::from_secret(key)).expect("Failed to create token");

//         let mut validation = Validation::new(Algorithm::HS512);
//         //validation.validate_exp = false;
//         let dec = decode::<Acc>(&token, &DecodingKey::from_secret(key), &validation)
//             .expect("Invalid key");
//         println!("help");
//     }
// }

// #[cfg(test)]
// mod EdTest {
//     //use ed25519::{Signature};
//     //use std::str::FromStr;
//     //use jsonwebtoken::jwk::PublicKeyUse::Signature;
//
//     // pub struct HelloSigner<S>
//     // where
//     //     S: Signer<ed25519::Signature>,
//     // {
//     //     pub signing_key: S,
//     // }
//     //
//     // impl<S> HelloSigner<S>
//     // where
//     //     S: Signer<ed25519::Signature>,
//     // {
//     //     pub fn sign(&self, person: &str) -> ed25519::Signature {
//     //         // NOTE: use `try_sign` if you'd like to be able to handle
//     //         // errors from external signing services/devices (e.g. HSM/KMS)
//     //         // <https://docs.rs/signature/latest/signature/trait.Signer.html#tymethod.try_sign>
//     //         self.signing_key.sign(format_message(person).as_bytes())
//     //     }
//     // }
//     //
//     // pub struct HelloVerifier<V> {
//     //     pub verifying_key: V,
//     // }
//     //
//     // impl<V> HelloVerifier<V>
//     // where
//     //     V: Verifier<ed25519::Signature>,
//     // {
//     //     pub fn verify(
//     //         &self,
//     //         person: &str,
//     //         signature: &ed25519::Signature,
//     //     ) -> Result<(), ed25519::Error> {
//     //         self.verifying_key
//     //             .verify(format_message(person).as_bytes(), signature)
//     //     }
//     // }
//     //
//     // fn format_message(person: &str) -> String {
//     //     format!("Hello, {}!", person)
//     // }
//     use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};
//
//     use rand::rngs::OsRng;
//
//     #[test]
//     fn ed() {
//         let mut csprng = OsRng;
//         let signing_key: SigningKey = SigningKey::generate(&mut csprng);
//
//         let test0 = String::from_utf8(signing_key.to_bytes().to_vec()).unwrap();
//
//         use ed25519_dalek::{Signature, Signer};
//         let message: &[u8] = b"This is a test of the tsunami alert system.";
//         let signature: Signature = signing_key.sign(message);
//
//         let test = signature.to_string();
//
//         assert!(signing_key.verify(message, &signature).is_ok());
//
//         let verifying_key: VerifyingKey = signing_key.verifying_key();
//         assert!(verifying_key.verify(message, &signature).is_ok());
//
//         //let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
//         // /// `HelloSigner` defined above instantiated with `ed25519-dalek` as
//         // /// the signing provider.
//         // pub type DalekHelloSigner = HelloSigner<ed25519_dalek::SigningKey>;
//         //
//         // let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
//         // let signer = DalekHelloSigner { signing_key };
//         // let person = "Joe"; // Message to sign
//         // let signature = signer.sign(person);
//         //
//         // /// `HelloVerifier` defined above instantiated with `ed25519-dalek`
//         // /// as the signature verification provider.
//         // pub type DalekHelloVerifier = HelloVerifier<ed25519_dalek::VerifyingKey>;
//         //
//         // let verifying_key: ed25519_dalek::VerifyingKey = signer.signing_key.verifying_key();
//         // let verifier = DalekHelloVerifier { verifying_key };
//         // assert!(verifier.verify(person, &signature).is_ok());
//     }
// }

pub async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("target/site/favicon.ico")?)
}

pub async fn hello() -> impl Responder {
    let index = read_to_string("artcord-builder/index.html").unwrap_or_default();
    HttpResponse::Ok().body(index)
    // HttpResponse::Ok().body("
    //     <!DOCTYPE html>
    //     <html>
    //     <head>
    //         <meta charset=\"utf-8\">
    //         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
    //         <title>ArtCord</title><link id=\"leptos\" href=\"/pkg/leptos_start5.css\" rel=\"stylesheet\" data-hk=\"0-0-0-13\">
    //         <link rel=\"modulepreload\" href=\"/pkg/leptos_start5.js\">
    //         <link rel=\"preload\" href=\"/pkg/leptos_start5_bg.wasm\" as=\"fetch\" type=\"application/wasm\" crossorigin=\"\">
    //         <script type=\"module\">
    //             function idle(c) {
    //                 if (\"requestIdleCallback\" in window) {
    //                     window.requestIdleCallback(c);
    //                 } else {
    //                     c();
    //                 }
    //             }
    //             idle(() => {

    //                 import('/pkg/leptos_start5.js')
    //                     .then(mod => {
    //                         console.log(\"dflopdfdf,l\");
    //                     mod.default('/pkg/leptos_start5_bg.wasm').then(() => mod.hydrate());
    //                 })
    //             });
    //         </script>
    //     </head>

    //     <body>
    //     </body>

    //         <script>__LEPTOS_PENDING_RESOURCES = [];__LEPTOS_RESOLVED_RESOURCES = new Map();__LEPTOS_RESOURCE_RESOLVERS = new Map();__LEPTOS_LOCAL_ONLY = [];</script>
    //     </html>
    // ")
}

use leptos::{get_configuration};

pub async fn create_server(galley_root_dir: Arc<String>, assets_root_dir: Arc<String>) -> Server {
    let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
    // let conf: ConfFile = ConfFile {
    //     leptos_options: LeptosOptions {
    //         output_name: "leptos_start5".to_string(),
    //         site_root: "target/site".to_string(),
    //         site_pkg_dir: "pkg".to_string(),
    //         env: DEV,
    //         site_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3000)),
    //         reload_port: 3001,
    //         reload_external_port: None,
    //         reload_ws_protocol: WS,
    //         not_found_path: "/404".to_string(),
    //         hash_file: String::from("hash.txt"),
    //         hash_files: true,
    //     },
    // };
    //info!("current leptos config is {:#?}", &conf);
    let addr = conf.leptos_options.site_addr;
    let routes = generate_route_list(artcord_leptos::app::App);
    info!("listening on http://{}", &addr);
    //let sessions = Arc::new(RwLock::new(HashMap::<uuid::Uuid, Addr<WsConnection>>::new()));

    //let galley_root_dir = galley_root_dir.to_string();
    //let assets_root_dir = assets_root_dir.to_string(); a
    let server = HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        // let site_root = &leptos_options.site_root;
        //println!("site root: {}", &*assets_root_dir);
        let pkg_url = format!("{}/pkg", &*assets_root_dir);
        //println!("pkg dir: {}", pkg_url);


        let mut app = App::new()
        .route("/favicon.ico", web::get().to(favicon))
        .service(Files::new("/assets/gallery", &*galley_root_dir))
        .service(Files::new("/assets", &*assets_root_dir))
        .service(Files::new("/pkg", pkg_url));

        cfg_if! {
            if #[cfg(feature = "csr")] {
                app = app.route("/{filename:.*}", web::get().to(hello));
            }
            else {
                app = app.leptos_routes(
                    leptos_options.to_owned(),
                    routes.to_owned(),
                    artcord_leptos::app::App,
                );
            }
        };


        
        

        

        // let create_prodution_app =  || {
            
        // };
        // let app = create_prodution_app();

        // let create_debug_app =  || {
            
        // };

        app
        //.route("/{filename:.*}", web::get().to(hello))
        // .leptos_routes(
        //     leptos_options.to_owned(),
        //     routes.to_owned(),
        //     artcord_leptos::app::App,
        // )
        //.route("/{filename:.*}", web::get().to(hello))
    })
    .workers(1)
    .bind(&addr)
    .unwrap()
    .run();

    #[cfg(debug_assertions)]
    {
        use tokio_tungstenite::connect_async;
        use tokio::sync::mpsc;
        use artcord_state::message::debug_client_msg::DebugClientMsg;
        use futures::SinkExt;
        use tokio_tungstenite::tungstenite::Message;
        use artcord_leptos_web_sockets::WsPackage;
        use artcord_state::message::debug_msg_key::DebugMsgPermKey;
        use artcord_state::message::debug_server_msg::DebugServerMsg;
        use artcord_leptos_web_sockets::WsRouteKey;
        use futures::future;
        use futures::pin_mut;

        

        

        

        let url = url::Url::parse("ws://localhost:3001").unwrap();

        let (send, mut recv) = mpsc::channel::<Message>(1);

        let ready_package = WsPackage::<u128, DebugMsgPermKey, DebugClientMsg> {
            key: WsRouteKey::Perm(DebugMsgPermKey::Debug),
            data: DebugClientMsg::RuntimeReady,
        };

        let ready_package = DebugClientMsg::as_vec(&ready_package);

        match ready_package {
            Ok(ready_package) => {
                let ready_package = Message::binary(ready_package);
                trace!("socekt: sending ready msg: {:?}", &ready_package);
                let send_result = send.send(ready_package).await;
                if let Err(err) = send_result {
                    error!("ws: failed to send ready msg: {}", err);
                }
            }
            Err(err) => {
                error!("ws: failed to serialize ready msg: {}", err);
            }
        }


        let (ws_stream, _) = connect_async(url).await.unwrap();
        let (mut write, read) = ws_stream.split();
        
        let read = {
            read.for_each_concurrent(100, |server_msg| async {
                match server_msg {
                    Ok(server_msg) => {
                        match server_msg {
                            tokio_tungstenite::tungstenite::Message::Binary(client_msg) => {
                                let client_msg = DebugServerMsg::from_bytes(&client_msg);
                                match client_msg {
                                    Ok(client_msg) => {
                                        trace!("ws: recv client msg: {:?}", client_msg);
                                    }
                                    Err(err) => {
                                        error!("ws: failed to deserialize client msg: {}", err);
                                    }
                                }
                            }
                            _ => {
                                warn!("ws: recv uwknown msg");
                            }
                        }
                    }
                    Err(err) => {
                        error!("ws: error on recv: {}", err);
                    }
                }
            })
        };

        let write = async move {
            while let Some(msg) = recv.recv().await {
                write.send(msg).await.unwrap();
            }
        };

        

        tokio::spawn(async move {
            pin_mut!(read, write);
            future::select(read, write).await;
        });
    }


    server
}
