use artcord_leptos_web_sockets::WsPackage;
use artcord_mongodb::database::DB;
use artcord_state::backend::{self, listener_tracker_send};
use artcord_state::global::{self, ThresholdTracker};
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
use futures::StreamExt;
use leptos::leptos_config::{ConfFile, Env};
use leptos::logging::warn;
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use futures::future::{ok, Either, MapOk, Ready};
use leptos::{get_configuration, IntoView, LeptosOptions};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{read_to_string, DirEntry};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::ops::Deref;
use tracing::{debug, error, info, trace};

use cfg_if::cfg_if;

use std::sync::Arc;

pub const TOKEN_SIZE: usize = 257;

// pub struct HttpServer<T: AsRef<str>, V: IntoView> {
//     pub tracker: TaskTracker,

// }

// pub struct ResData<T: AsRef<str> + std::marker::Send, V: IntoView, F: Fn() -> V + Clone + std::marker::Send + 'static >

pub struct Http <T: AsRef<str> + std::marker::Send + std::marker::Sync + 'static> {
    pub res_data: Arc<ResData<T>>,
    pub tacker: TaskTracker,
    pub default_threshold: global::DefaultThreshold,
    pub db: Arc<DB>,
}

pub struct ResData<T: AsRef<str> + std::marker::Send + std::marker::Sync + 'static> {
    pub leptos_options: leptos::leptos_config::LeptosOptions,
    pub assets_res: HashMap<String, Vec<u8>>,
    pub not_found_res: Vec<u8>,
    pub forbidden_res: Vec<u8>,
    pub index_res: Vec<u8>,
    pub schemas: Vec<T>,
    pub asset_dir: std::path::PathBuf,
    pub csr_index: String,
}

pub struct HttpIp {
    pub banned_until: global::BanType,
    pub block_tracker: global::ThresholdTracker,
    pub ban_tracker: global::ThresholdTracker,
}

pub async fn create_server<
    TimeMiddlewareType: global::TimeMiddleware + Clone + Sync + Send + 'static,
    SocketAddrMiddlewareType: global::GetUserAddrMiddleware + Send + Sync + Clone + 'static,
>(
    http_addr: String,
    db: Arc<DB>,
    default_threshold: global::DefaultThreshold,
    cancelation_token: CancellationToken,
    galley_root_dir: String,
    assets_dir: String,
    csr_index: String,
    ws_tx: mpsc::Sender<backend::WsMsg>,
    mut http_rx: mpsc::Receiver<backend::HttpMsg>,
    time_middleware: TimeMiddlewareType,
    socket_middleware: SocketAddrMiddlewareType,
) {
    let conf = get_configuration(Some("Cargo.toml"))
        .await
        .unwrap_or_else(|_| {
            warn!("leptops config in Cargo.toml was not found");
            ConfFile {
                leptos_options: LeptosOptions {
                    output_name: "leptos_start5".to_string(),
                    site_root: "target/site".to_string(),
                    site_pkg_dir: "pkg".to_string(),
                    env: Env::DEV,
                    site_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3000)),
                    reload_port: 3001,
                    reload_external_port: None,
                    reload_ws_protocol: leptos::leptos_config::ReloadWSProtocol::WS,
                    not_found_path: "/404".to_string(),
                    hash_file: String::from("hash.txt"),
                    hash_files: true,
                },
            }
        });

    let leptos_options = conf.leptos_options;
    let app_fn = move || leptos::view! { <artcord_leptos::app::App/> };

    let (routes, static_data_map) = leptos_router::generate_route_list_inner_with_context(
        { move || leptos::IntoView::into_view(app_fn()) },
        || {},
    );

    let schemas: Vec<String> = static_data_map.into_iter().map(|(key, v)| key).collect();

    let listener = tokio::net::TcpListener::bind(http_addr )
        .await
        .unwrap();

    let not_found_res = "HTTP/1.1 404 Not Found\r\n\r\n";
    let not_found_res = not_found_res.as_bytes();

    let forbidden_res = "HTTP/1.1 403 forbidden\r\n\r\n";
    let forbidden_res = forbidden_res.as_bytes();

    
    let index_res: Vec<u8> = get_asset(&csr_index, "html").await.unwrap();

    let assets_res = get_assets_res(&assets_dir).await;
    let k = assets_res
        .iter()
        .map(|(k, _)| k.clone())
        .collect::<Vec<String>>();
    debug!("AAAAAAAA: {:#?}", k);


    let res_data = ResData {
        leptos_options: leptos_options,
        assets_res: assets_res,
        index_res: index_res,
        not_found_res: not_found_res.to_vec(),
        forbidden_res: forbidden_res.to_vec(),
        schemas: schemas,
        asset_dir: std::path::PathBuf::from(assets_dir),
        csr_index,
        //ips: HashMap::new(),
    };
    let res_data = Arc::new(res_data);
    //let tacker = TaskTracker::new();
    // let mut block_tracker = global::ThresholdTracker::new(time);
    // let mut ban_tracker = global::ThresholdTracker::new(time);
    //let mut banned_until: global::BanType = None;
    let mut http_ips: HashMap<core::net::IpAddr, HttpIp> = HashMap::new();
    let mut listener_tracker: backend::ListenerTrackerType = HashMap::new();

    // let d = global::double_throttle(
    //     &mut block_tracker,
    //     &mut ban_tracker,
    //     block_threshold,
    //     ban_threshold,
    //     ban_reason,
    //     ban_duration,
    //     &time,
    //     &mut banned_until,
    // );

    let http = Http {
        res_data: res_data,
        default_threshold,
        tacker: TaskTracker::new(),
        db: db.clone(),
    };

    #[cfg(feature = "development")]
    {
        use artcord_state::global;
        use futures::future;
        use futures::pin_mut;
        use futures::SinkExt;
        use tokio::sync::mpsc;
        use tokio_tungstenite::connect_async;
        use tokio_tungstenite::tungstenite::Message;

        let url = url::Url::parse("ws://localhost:3001").unwrap();

        let (send, mut recv) = mpsc::channel::<Message>(1);

        let ready_package: WsPackage<global::DebugClientMsg> =
            (0, global::DebugClientMsg::RuntimeReady);

        let ready_package = global::DebugClientMsg::as_vec(&ready_package);

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
                    Ok(server_msg) => match server_msg {
                        tokio_tungstenite::tungstenite::Message::Binary(client_msg) => {
                            let client_msg = global::DebugServerMsg::from_bytes(&client_msg);
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
                    },
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
    
    loop {
        select! {
            result = listener.accept() => {
                let (stream, addr) = match result {
                    Ok(v) => v,
                    Err(err) => {
                        debug!("tcp accept err: {err}");
                        continue;
                    }
                };
                let addr = socket_middleware.get_addr(addr).await;
                let time = time_middleware.get_time().await;

                let result = on_con(&http, &ws_tx, &mut http_ips, &mut listener_tracker, stream, addr, time).await;
                if let Err(err) = result {
                    debug!("on_con: {}", err);
                }
                // let allow = match allow {
                //     Ok(allow) => allow,
                //     Err(err) => {
                //         debug!("firewall err: {}", err);
                //         false
                //     },
                // };

                // if allow {
                //     tacker.spawn(); 
                // }
                // let res_data = res_data.clone();
                // tacker.spawn(async move {
                //     stream;
                //     res_data;
                // });
            }
            msg = http_rx.recv() => {
                let Some(msg) = msg else {
                    error!("http_tx closed");
                    break;
                };
                let result = on_msg(msg,  &mut http_ips, &mut listener_tracker).await;
                if let Err(err) = result {
                    error!("on_msg: {err}");
                }
            }
            _ = cancelation_token.cancelled() => {
                break;
            }
        }
    }

  

    http.tacker.close();
    http.tacker.wait().await;
}

async fn on_con <T: AsRef<str> + std::marker::Send + std::marker::Sync + 'static>(
    http: &Http<T>,
    ws_tx: &mpsc::Sender<backend::WsMsg>,
    http_ips: &mut std::collections::HashMap<core::net::IpAddr, HttpIp>,
    listener_tracker: &mut backend::ListenerTrackerType,
    mut stream: tokio::net::TcpStream,
    socket_addr: std::net::SocketAddr,
    time: DateTime<Utc>,
) -> Result<(), OnConErr> {
    let ip = socket_addr.ip();
    let http_ip = match http_ips.get_mut(&ip) {
        Some(ip) => ip,
        None => {
            let http_ip = match http.db.http_ip_find_one_by_ip(ip).await? {
                Some(saved_http_ip) => HttpIp {
                    block_tracker: saved_http_ip.block_tracker,
                    ban_tracker: saved_http_ip.ban_tracker,
                    banned_until: saved_http_ip.banned_until,
                },
                None => HttpIp {
                    block_tracker: ThresholdTracker::new(time),
                    ban_tracker: ThresholdTracker::new(time),
                    banned_until: None,
                },
            };
            http_ips.entry(ip).or_insert(http_ip)
        }
    };

    let access = global::double_throttle(
        &mut http_ip.block_tracker,
        &mut http_ip.ban_tracker,
        &http.default_threshold.ws_http_block_threshold,
        &http.default_threshold.ws_http_ban_threshold,
        &http.default_threshold.ws_http_ban_reason,
        &http.default_threshold.ws_http_ban_duration,
        &time,
        &mut http_ip.banned_until,
    );

    match access {
        global::AllowCon::Allow => {
            http.tacker.spawn(handle_res_ok(stream, http.res_data.clone()));
            listener_tracker_send(listener_tracker, global::ServerMsg::HttpLiveStatsConAllowed { ip, total_amount: 0 }).await?;
            //http.tacker.spawn(handle_res_ok(stream, http.res_data.clone()));
        }  
        global::AllowCon::AlreadyBanned => {
            
        }
        global::AllowCon::Blocked => {
            http.tacker.spawn(handle_res_block(stream, http.res_data.clone()));
            listener_tracker_send(listener_tracker, global::ServerMsg::HttpLiveStatsConBlocked { ip, total_amount: 0 }).await?;
        }
        global::AllowCon::UnbannedAndAllow => {
            http.tacker.spawn(handle_res_ok(stream, http.res_data.clone()));
            listener_tracker_send(listener_tracker, global::ServerMsg::HttpLiveStatsConAllowed { ip, total_amount: 0 }).await?;
        }
        global::AllowCon::UnbannedAndBlocked => {
            http.tacker.spawn(handle_res_block(stream, http.res_data.clone()));
            listener_tracker_send(listener_tracker, global::ServerMsg::HttpLiveStatsConBlocked { ip, total_amount: 0 }).await?;
        }
        global::AllowCon::Banned((date, reason)) => {
           // let (done_tx, done_rx) = oneshot::channel::<()>();
            ws_tx.send(backend::WsMsg::Ban { ip, date, reason }).await?;
        //    done_rx.await.map_err(|_| OnConErr::RxOnBan)?;
            listener_tracker_send(listener_tracker, global::ServerMsg::HttpLiveStatsConBanned { ip, total_amount: 0 }).await?;
        }
    };

    Ok(())
}

async fn on_msg(msg: backend::HttpMsg, http_ips: &mut std::collections::HashMap<core::net::IpAddr, HttpIp>, listener_tracker: &mut backend::ListenerTrackerType) -> Result<(), HttpOnMsgErr> {
    match msg {
        backend::HttpMsg::Ban { ip, date, reason, done_tx } => {
            if let Some(http_ip) = http_ips.get_mut(&ip) {
                http_ip.banned_until = Some((date, reason));

            } else {
                error!("on_ban ip '{}' not found", ip);
            }

            done_tx.send(()).map_err(|_| HttpOnMsgErr::TxOnBan)?;
        }
        backend::HttpMsg::AddListener { con_id, con_tx, ws_key, done_tx } => {
            listener_tracker.insert(con_id, (ws_key, con_tx));
            done_tx.send(()).map_err(|_| HttpOnMsgErr::TxOnAddListener)?;
        }
        backend::HttpMsg::RemoveListener { con_id } => {
            listener_tracker.remove(&con_id);
        }
    }

    Ok(())
}

async fn handle_res_block<T: AsRef<str> + std::marker::Send + std::marker::Sync + 'static>(
    mut stream: tokio::net::TcpStream,
    res_data: Arc<ResData<T>>,
) {
    let mut buff: [u8; 8192] = [0; 8192];
    let size = tokio::io::AsyncReadExt::read(&mut stream, &mut buff).await;

    let size = match size {
        Ok(size) => size,
        Err(err) => {
            debug!("tcp read err: {err}");
            return;
        }
    };

    let result = stream.write_all(&res_data.forbidden_res).await;
    if let Err(err) = result {
        debug!("writing to stream err: {}", err);
    }
}

async fn handle_res_ok<T: AsRef<str> + std::marker::Send + std::marker::Sync + 'static>(
    mut stream: tokio::net::TcpStream,
    res_data: Arc<ResData<T>>,
) {
    let mut buff: [u8; 8192] = [0; 8192];
    let size = tokio::io::AsyncReadExt::read(&mut stream, &mut buff).await;

    let size = match size {
        Ok(size) => size,
        Err(err) => {
            debug!("tcp read err: {err}");
            return;
        }
    };

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    let status = req.parse(&buff);
    let status = match status {
        Ok(status) => status,
        Err(err) => {
            debug!("err {}", err);
            return;
        }
    };

    let Some(path) = req.path else {
        debug!("path not included");
        return;
    };
    trace!("connected: {} ", path);
    let full_path = ["http://leptos.dev", path].concat();

    //let app_fn = (*res_data.app_fn)();e  
    
   
    
    

    cfg_if! {
        if #[cfg(feature = "development")] {
            let new_path = res_data.asset_dir.join(if path.len() > 1 && path.starts_with('/') { &path[1..] } else { path });
            trace!("ASSET DIR: {:?} + {:?} = {:?}", res_data.asset_dir, path, new_path);
            
            let extension = match new_path.extension().map(|ex| ex.to_str().unwrap()   ) {
                Some(ex) => {
                    Some(get_asset(new_path.to_str().unwrap(), ex).await)
                }
                None => None
            };
           
            let result = if let Some(Ok(res)) = extension {
                stream.write_all(&res).await
            } else {
                let found = compare_path(path, &res_data.schemas);
                if found {
                    cfg_if! {
                        if #[cfg(feature = "serve_csr")] {
                            trace!("sending csr app....");
                            let index_res: Vec<u8> = get_asset(&res_data.csr_index, "html").await.unwrap();
                            stream.write_all(&index_res).await
                        } else {
                            trace!("rendering app....");
                            let app = render_my_app(&res_data.leptos_options, &full_path).await;
                            stream.write_all(app.as_bytes()).await
                        }
                    }
                } else {
                    stream.write_all(&res_data.not_found_res).await
                }
            };
        
            if let Err(err) = result {
                debug!("writing to stream err: {}", err);
            }
           
        } else {
            let result = if let Some(res) = res_data.assets_res.get(path) {
                stream.write_all(res).await
            } else {
                let found = compare_path(path, &res_data.schemas);
                if found {
                    cfg_if! {
                        if #[cfg(feature = "serve_csr")] {
                            trace!("sending csr app....");
                            stream.write_all(&res_data.index_res).await
                        } else {
                            trace!("rendering app....");
                            let app = render_my_app(&res_data.leptos_options, &full_path).await;
                            stream.write_all(app.as_bytes()).await
                        }
                    }
                } else {
                    stream.write_all(&res_data.not_found_res).await
                }
            };
            if let Err(err) = result {
                debug!("writing to stream err: {}", err);
            }
        }
    }

    
}

async fn get_assets_res(
    root_dir: &str,
) -> HashMap<String, Vec<u8>> {
    //let assets_dir = assets_dir.to_string();
    let mut responses: HashMap<String, Vec<u8>> = HashMap::new();
    // debug!("reading {}", assets_dir);
    // let mut dir = tokio::fs::read_dir(&assets_dir).await.unwrap(); 
    // let mut last: Option<std::fs::DirEntry> = None;

    let mut queue = std::collections::VecDeque::from([String::new()]);

    while let Some(path) = queue.pop_front() {
        let dir_path =std::path::Path::new(&root_dir).join(&path);
        let mut dir = tokio::fs::read_dir(&dir_path).await.unwrap();
        while let Some(entry) = dir.next_entry().await.unwrap() {

            let kind = entry.file_type().await.unwrap();
            if kind.is_dir() {
                let sub_assets_dir = std::path::Path::new(&path).join(entry.file_name());
                let sub_assets_dir = sub_assets_dir.to_str().unwrap();
                trace!("reading: {sub_assets_dir}");
                queue.push_back(sub_assets_dir.to_string());
            } else if kind.is_file() {
                let name = entry.file_name();
                let name = name.to_str().unwrap();
                let Some(extension) = std::path::Path::new(name)
                    .extension()
                    .map(|v| v.to_str())
                    .flatten()
                else {
                    continue;
                };
                
                let asset_path = dir_path.join(name);
    
                match get_asset(asset_path.to_str().unwrap(), extension).await {
                    Ok(asset) => {
                        let route = std::path::Path::new("/").join(&path).join(name);
                        responses.insert(route.to_str().unwrap().to_string(), asset);
                    }
                    Err(err) => {
                        debug!("getting asset err: {}", err);
                    }
                }
                
            }

        }

        


    }

    responses
}

async fn get_asset<'a>(path: &str, extension: &'a str) -> Result<Vec<u8>, GetAssetErr<'a>> {
    trace!("reading: {path}");

    let file: Vec<u8> = tokio::fs::read(path).await.map_err(|err| GetAssetErr::IO(path.to_string(), err))?;
    let file_length = file.len();
    let content_type = get_content_type(extension).ok_or_else(|| GetAssetErr::UnSupportedFileType(extension))?;
    let header = format!("HTTP/1.1 200 OK\r\ncontent-type: {content_type}\r\ncontent-length: {file_length}\r\n\r\n");
    let header = header.as_bytes();
    let res: Vec<u8> = [header, &file].concat();
    Ok(res)
}

fn get_content_type(extension: &str) -> Option<&'static str> {
    Some(
        match extension {
            "ico" => "x-icon",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "css" => "text/css",
            "js" => "text/javascript",
            "wasm" => "application/wasm",
            "html" => "text/html;charset=UTF-8",
            _ => {
                return None;
            }
        }
    )
}




// fn get_assets_res(
//     assets_dir: &str,
// ) -> std::pin::Pin<std::boxed::Box<dyn futures::Future<Output = HashMap<String, Vec<u8>  > >>> {
//     let assets_dir = assets_dir.to_string();
//     std::boxed::Box::pin(async move {
//         let mut responses: HashMap<String, Vec<u8>> = HashMap::new();
//         debug!("reading {}", assets_dir);
//         let mut dir = tokio::fs::read_dir(&assets_dir).await.unwrap();

//         while let Some(entry) = dir.next_entry().await.unwrap() {
//             let kind = entry.file_type().await.unwrap();
//             if kind.is_dir() {
//                 let name = entry.file_name();
//                 let name = name.to_str().unwrap();
//                 let sub_assets_dir = format!("{}/{}", assets_dir, name);
//                 let sub_responses = get_assets_res(&sub_assets_dir).await;
//                 for (sub_key, sub_data) in sub_responses {
//                     responses.insert(format!("/{}{}", name, sub_key), sub_data);
//                 }
//             } else if kind.is_file() {
//                 let name = entry.file_name();
//                 let name = name.to_str().unwrap();
//                 let Some(extension) = std::path::Path::new(name)
//                     .extension()
//                     .map(|v| v.to_str())
//                     .flatten()
//                 else {
//                     continue;
//                 };
//                 let new_path = std::path::Path::new(&assets_dir).join(name);
//                 let new_path = new_path.to_str().unwrap();

//                 match extension {
//                     "ico" => {
//                         let bytes = tokio::fs::read(new_path).await.unwrap();
//                         let bytes_content_length = bytes.len();
//                         let header = format!(
//                             "HTTP/1.1 200 OK\r\ncontent-type: x-icon\r\ncontent-length: {bytes_content_length}\r\n\r\n"
//                         );
//                         let header = header.as_bytes();
//                         let res: Vec<u8> = [header, &bytes].concat();
//                         let route = format!("/{}", name);
//                         responses.insert(route, res);
//                     }
//                     "webp" => {
//                         let bytes = tokio::fs::read(new_path).await.unwrap();
//                         let bytes_content_length = bytes.len();
//                         let header = format!(
//                             "HTTP/1.1 200 OK\r\ncontent-type: image/webp\r\ncontent-length: {bytes_content_length}\r\n\r\n"
//                         );
//                         let header = header.as_bytes();
//                         let res: Vec<u8> = [header, &bytes].concat();
//                         let route = format!("/{}", name);
//                         responses.insert(route, res);
//                     }
//                     "svg" => {
//                         let bytes = tokio::fs::read(new_path).await.unwrap();
//                         let bytes_content_length = bytes.len();
//                         let header = format!(
//                             "HTTP/1.1 200 OK\r\ncontent-type: image/svg+xml\r\ncontent-length: {bytes_content_length}\r\n\r\n"
//                         );
//                         let header = header.as_bytes();
//                         let res: Vec<u8> = [header, &bytes].concat();
//                         let route = format!("/{}", name);
//                         responses.insert(route, res);
//                     }
//                     "css" => {
//                         let css = tokio::fs::read(new_path).await.unwrap();
//                         let css_content_length = css.len();
//                         let css_header = format!(
//                             "HTTP/1.1 200 OK\r\ncontent-type: text/css\r\ncontent-length: {css_content_length}\r\n\r\n"
//                         );
//                         let css_header = css_header.as_bytes();
//                         let css_res: Vec<u8> = [css_header, &css].concat();
//                         let route = format!("/{}", name);
//                         responses.insert(route, css_res);
//                     }
//                     "js" => {
//                         let js: Vec<u8> = tokio::fs::read(new_path).await.unwrap();
//                         let js_content_length = js.len();
//                         let js_header = format!("HTTP/1.1 200 OK\r\ncontent-type: text/javascript\r\ncontent-length: {js_content_length}\r\n\r\n");
//                         let js_header = js_header.as_bytes();
//                         let js_res: Vec<u8> = [js_header, &js].concat();
//                         let route = format!("/{}", name);
//                         responses.insert(route, js_res);
//                     }
//                     "wasm" => {
//                         let wasm = tokio::fs::read(new_path).await.unwrap();
//                         let wasm_content_length = wasm.len();
//                         let wasm_header = format!("HTTP/1.1 200 OK\r\ncontent-type: application/wasm\r\ncontent-length: {wasm_content_length}\r\n\r\n");
//                         let wasm_header = wasm_header.as_bytes();
//                         let wasm_res = [wasm_header, &wasm].concat();

//                         // let not_found_res = "HTTP/1.1 404 Not Found\r\n\r\n";
//                         // let not_found_res = not_found_res.as_bytes().to_vec();

//                         let route = format!("/{}", name);
//                         responses.insert(route, wasm_res);
//                     }
//                     _ => {}
//                 }
//             }
//         }
//         responses
//     })
// }

async fn render_my_app(
    leptos_options: &leptos::leptos_config::LeptosOptions,
    path: &str,
) -> String {
    leptos_dom::HydrationCtx::reset_id();

    let runtime = leptos_reactive::create_runtime();

    let prefix: String = leptos_meta::generate_head_metadata_separated().1.into();

    let integration = leptos_router::ServerIntegration {
        path: path.to_string(),
    };
    leptos::provide_context(leptos_router::RouterIntegrationContext::new(integration));
    leptos::provide_context(leptos_meta::MetaContext::new());
    leptos::provide_context(leptos_options.clone());
    leptos::provide_context(leptos_router::Method::Get);

    let body = (leptos::view! { <artcord_leptos::app::App/> }).render_to_string();

    // let (bundle, runtime) = {
    //     let leptos_options = leptos_options.clone();
    //     let integration = leptos_router::ServerIntegration {
    //         path: path.to_string(),
    //     };

    //     leptos::leptos_dom::ssr::render_to_stream_with_prefix_undisposed_with_context_and_block_replacement(
    //         move || {
    //             leptos::provide_context(leptos_router::RouterIntegrationContext::new(integration));
    //             leptos::provide_context(leptos_meta::MetaContext::new());

    //             app_fn().into_view()
    //         },
    //         || leptos_meta::generate_head_metadata_separated().1.into(),
    //         || {
    //             leptos::provide_context(leptos_options);
    //             leptos::provide_context(leptos_router::Method::Get);
    //         },
    //         false,
    //     )
    // };

    // let mut shell = Box::pin(bundle);

    // let mut body = String::new();

    // while let Some(chunk) = shell.next().await {
    //     body.push_str(&chunk);
    // }

    let resources = leptos_reactive::SharedContext::pending_resources();
    let pending_resources = serde_json::to_string(&resources).unwrap();
    // let pending_fragments = leptos_reactive::SharedContext::pending_fragments();
    // let serializers = leptos_reactive::SharedContext::serialization_resolvers();
    let nonce_str = leptos_dom::nonce::use_nonce()
        .map(|nonce| format!(" nonce=\"{nonce}\""))
        .unwrap_or_default();
    let local_only = leptos_reactive::SharedContext::fragments_with_local_resources();
    let local_only = serde_json::to_string(&local_only).unwrap();

    let resolvers = format!(
        "<script{nonce_str}>__LEPTOS_PENDING_RESOURCES = \
            {pending_resources};__LEPTOS_RESOLVED_RESOURCES = new \
            Map();__LEPTOS_RESOURCE_RESOLVERS = new \
            Map();__LEPTOS_LOCAL_ONLY = {local_only};</script>"
    );

    let (head, tail) = leptos_integration_utils::html_parts_separated(
        &leptos_options,
        leptos::use_context::<leptos_meta::MetaContext>().as_ref(),
    );

    runtime.dispose();

    let app_content_length = head.len() + body.len() + resolvers.len() + tail.len();
    format!("HTTP/1.1 200 OK\r\ncontent-type: text/html\r\ncontent-length: {app_content_length}\r\n\r\n{head}{prefix}{body}{resolvers}{tail}")
}

async fn leptos_ssr(
    view: impl FnOnce() -> leptos_dom::View + 'static,
    prefix: impl FnOnce() -> leptos_reactive::Oco<'static, str> + 'static,
    additional_context: impl FnOnce() + 'static,
    replace_blocks: bool,
) -> String {
    leptos_dom::HydrationCtx::reset_id();

    // create the runtime
    let runtime = leptos_reactive::create_runtime();

    // Add additional context items
    additional_context();

    // the actual app body/template code
    // this does NOT contain any of the data being loaded asynchronously in resources
    let shell = view().render_to_string();

    //let resources = leptos_reactive::SharedContext::pending_resources();
    // let pending_resources = serde_json::to_string(&resources).unwrap();
    // let pending_fragments = leptos_reactive::SharedContext::pending_fragments();
    // let serializers = leptos_reactive::SharedContext::serialization_resolvers();
    // let nonce_str = leptos_dom::nonce::use_nonce()
    //     .map(|nonce| format!(" nonce=\"{nonce}\""))
    //     .unwrap_or_default();

    // let local_only = leptos_reactive::SharedContext::fragments_with_local_resources();
    // let local_only = serde_json::to_string(&local_only).unwrap();

    // let mut blocks = Vec::new();
    // let fragments = Vec::new();

    // for (fragment_id, data) in pending_fragments {
    //     if data.should_block {
    //         blocks
    //             .push((fragment_id, data.out_of_order.await));
    //     } else {
    //         fragments.push((fragment_id, data.out_of_order.await));
    //     }
    // }

    // let mut output: String = String::new();

    // {
    //     let nonce_str = nonce_str.clone();

    //     let resolvers = format!(
    //         "<script{nonce_str}>__LEPTOS_PENDING_RESOURCES = \
    //          {pending_resources};__LEPTOS_RESOLVED_RESOURCES = new \
    //          Map();__LEPTOS_RESOURCE_RESOLVERS = new \
    //          Map();__LEPTOS_LOCAL_ONLY = {local_only};</script>"
    //     );

    //     if replace_blocks {

    //         let prefix = prefix();

    //         let mut shell = shell;

    //         for (blocked_id, blocked_fragment) in blocks {
    //             let open = format!("<!--suspense-open-{blocked_id}-->");
    //             let close =
    //                 format!("<!--suspense-close-{blocked_id}-->");
    //             let (first, rest) =
    //                 shell.split_once(&open).unwrap_or_default();
    //             let (_fallback, rest) =
    //                 rest.split_once(&close).unwrap_or_default();

    //             shell =
    //                 format!("{first}{blocked_fragment}{rest}").into();
    //         }

    //         format!("{prefix}{shell}{resolvers}");
    //     } else {
    //         let mut blocking = blocks.into_iter().map(|b| fragments_to_chunks(nonce_str.clone(), b)).collect::<String>();
    //         let prefix = prefix();
    //         format!("{prefix}{shell}{resolvers}{blocking}");
    //     }
    // }

    //let mut blocking = blocks.into_iter().map(|b| fragments_to_chunks(nonce_str.clone(), b)).collect::<String>();
    let prefix = prefix();
    // format!("{prefix}{shell}{resolvers}{blocking}")
    format!("{prefix}{shell}")
}

fn fragments_to_chunks(nonce_str: String, (fragment_id, html): (String, String)) -> String {
    format!(
        r#"
                <template id="{fragment_id}f">{html}</template>
                <script{nonce_str}>
                    (function() {{ let id = "{fragment_id}";
                    let open = undefined;
                    let close = undefined;
                    let walker = document.createTreeWalker(document.body, NodeFilter.SHOW_COMMENT);
                    while(walker.nextNode()) {{
                         if(walker.currentNode.textContent == `suspense-open-${{id}}`) {{
                           open = walker.currentNode;
                         }} else if(walker.currentNode.textContent == `suspense-close-${{id}}`) {{
                           close = walker.currentNode;
                         }}
                      }}
                    let range = new Range();
                    range.setStartAfter(open);
                    range.setEndBefore(close);
                    range.deleteContents();
                    let tpl = document.getElementById("{fragment_id}f");
                    close.parentNode.insertBefore(tpl.content.cloneNode(true), close);}})()
                </script>
                "#
    )
}

fn compare_path<T: AsRef<str>>(path: &str, path_schemas: &[T]) -> bool {
    'schema_loop: for schema_path in path_schemas {
        let mut schema_chars = schema_path.as_ref().chars().peekable();
        let mut path_chars = path.chars().peekable();
        let mut skip = false;

        if path_chars.peek().map(|c| *c == '/').unwrap_or(false)
            && schema_chars.peek().map(|c| *c != '/').unwrap_or(false)
        {
            path_chars.next();
        } else if path_chars.peek().map(|c| *c != '/').unwrap_or(false)
            && schema_chars.peek().map(|c| *c == '/').unwrap_or(false)
        {
            schema_chars.next();
        }

        'path_loop: loop {
            let Some(path_char) = path_chars.next() else {
                let Some(schema_char) = schema_chars.next() else {
                    break;
                };

                if skip {
                    if schema_char == '/' {
                        continue 'schema_loop;
                    }

                    while let Some(schema_char) = schema_chars.next() {
                        if schema_char == '/' {
                            continue 'schema_loop;
                        }
                    }

                    break;
                } else if schema_char == '/' && schema_chars.peek().is_none() {
                    break;
                } else {
                    continue 'schema_loop;
                }
            };

            let Some(schema_char) = schema_chars.next() else {
                if skip {
                    if path_char == '/' {
                        continue 'schema_loop;
                    }

                    while let Some(path_char) = path_chars.next() {
                        if path_char == '/' {
                            continue 'schema_loop;
                        }
                    }

                    break;
                } else if path_char == '/' && path_chars.peek().is_none() {
                    break;
                } else {
                    continue 'schema_loop;
                }
            };

            match schema_char {
                '/' => {
                    skip = false;
                    if path_char != '/' {
                        while let Some(path_char) = path_chars.next() {
                            if path_char == '/' {
                                continue 'path_loop;
                            }
                        }

                        continue 'schema_loop;
                    }

                    continue;
                }
                ':' => {
                    skip = true;
                }
                schema_char => {
                    if path_char == '/' {
                        while let Some(schema_char) = schema_chars.next() {
                            if schema_char == '/' {
                                skip = false;
                                continue 'path_loop;
                            }
                        }

                        if skip && path_chars.peek().is_none() {
                            break;
                        }

                        continue 'schema_loop;
                    }
                    if !skip && schema_char != path_char {
                        continue 'schema_loop;
                    }
                }
            }
        }

        return true;
    }
    false
}


#[derive(thiserror::Error, Debug)]
pub enum GetAssetErr<'a> {
    #[error("error reading {0} : {1}")]
    IO(String, std::io::Error),

    #[error("failed to get content_type for extension {0}, reason: unsupported")]
    UnSupportedFileType(&'a str),
}

#[derive(thiserror::Error, Debug)]
pub enum HttpOnMsgErr {
    // #[error("on_ban ip '{0}' doesnt exist")]
    // OnBanIpNotFound(IpAddr),

    #[error("on_ban failed to send done_tx")]
    TxOnBan,

    #[error("on_add_listener failed to send done_tx")]
    TxOnAddListener,

   
}

#[derive(thiserror::Error, Debug)]
pub enum OnConErr {
    #[error("on_ban failed to receive done_tx")]
    RxOnBan,

    #[error("listener tracker err: {0}")]
    ListenerTrackerErr(#[from] artcord_state::backend::ListenerTrackerErr),

    #[error("db error: {0}")]
    DBError(#[from] artcord_mongodb::database::DBError),


    #[error("failed to send msg to ws: {0}")]
    WsTx(#[from] mpsc::error::SendError<artcord_state::backend::WsMsg>),
}

#[cfg(test)]
mod tests {
    use crate::server::compare_path;

    #[test]
    fn compare() {
        assert!(compare_path("/user/69/profile", &["/user/:id/profile"]));
        assert!(compare_path("/user/a/profile", &["/user/:id/profile/"]));
        assert!(compare_path("/user/a/profile/", &["/user/:id/profile/"]));
        assert!(compare_path("/user/a/profile/", &["/user/:id/profile"]));
        assert!(compare_path(
            "/user/profile/profile",
            &["/user/:id/profile"]
        ));
        assert!(!compare_path("/user2/profile", &["/user/:id/profile"]));

        assert!(!compare_path("/user/profile/", &["/user/profile/:id"]));
        assert!(compare_path("/user/profile/a", &["/user/profile/:id"]));
        assert!(compare_path("/123", &[":id"]));
        assert!(!compare_path("/", &[":id"]));
        assert!(compare_path("/123/aaa", &[":id/aaa"]));
        assert!(!compare_path("/123/aab", &[":id/aaa"]));

        assert!(!compare_path("/", &["/one", "/user/:id/profile"]));
        assert!(!compare_path("", &["/one", "/user/:id/profile"]));

        assert!(compare_path("/", &["/", "/user/:id/profile"]));
        assert!(compare_path("", &["/", "/user/:id/profile"]));
        assert!(compare_path("/", &["", "/user/:id/profile"]));

        assert!(compare_path("/user/123", &["/user/123/"]));
        assert!(compare_path("/user/123", &["/user/123"]));
        assert!(!compare_path("/user/123", &["/user/1234"]));
        assert!(!compare_path("/user/123", &["/user/123/4"]));

        assert!(compare_path("/user/123/", &["/user/123"]));
        assert!(compare_path("/user/123", &["/user/123"]));
        assert!(!compare_path("/user/1234", &["/user/123"]));
        assert!(!compare_path("/user/123/4", &["/user/123"]));

        assert!(compare_path("/user/123/", &["/user/123/"]));
        assert!(compare_path("/user/123", &["/user/123/"]));
        assert!(!compare_path("/user/1234", &["/user/123/"]));
        assert!(!compare_path("/user/123/4", &["/user/123/"]));
    }
}
