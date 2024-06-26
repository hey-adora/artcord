use artcord_leptos_web_sockets::WsPackage;
use artcord_state::global;
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
use futures::StreamExt;
use leptos::leptos_config::{ConfFile, Env};
use leptos::logging::warn;
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use futures::future::{ok, Either, MapOk, Ready};
use leptos::{get_configuration, IntoView, LeptosOptions};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::ops::Deref;
use tracing::{debug, error, info, trace};

use cfg_if::cfg_if;

use std::sync::Arc;

pub const TOKEN_SIZE: usize = 257;

// pub struct HttpServer<T: AsRef<str>, V: IntoView> {
//     pub tracker: TaskTracker,

// }

// pub struct ResData<T: AsRef<str> + std::marker::Send, V: IntoView, F: Fn() -> V + Clone + std::marker::Send + 'static >

pub struct HttpIp {
    pub block_tracker: global::ThresholdTracker,
    pub ban_tracker: global::ThresholdTracker,
}

pub struct ResData<T: AsRef<str> + std::marker::Send + 'static> {
    pub leptos_options: leptos::leptos_config::LeptosOptions,
    pub assets_res: HashMap<String, Vec<u8>>,
    pub not_found_res: Vec<u8>,
    pub index_res: Vec<u8>,
    pub schemas: Vec<T>,
}

pub async fn create_server(
    cancelation_token: CancellationToken,
    galley_root_dir: &str,
    assets_root_dir: &str,
    default_threshold: global::DefaultThreshold,
    time: DateTime<Utc>,
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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    let not_found_res = "HTTP/1.1 404 Not Found\r\n\r\n";
    let not_found_res = not_found_res.as_bytes();

    let index_bytes = tokio::fs::read("./artcord-http/index.html").await.unwrap();
    let index_content_length = index_bytes.len();
    let index_header = format!(
        "HTTP/1.1 200 OK\r\ncontent-type: text/html;charset=UTF-8\r\ncontent-length: {index_content_length}\r\n\r\n"
    );
    let index_header = index_header.as_bytes();
    let index_res: Vec<u8> = [index_header, &index_bytes].concat();

    let assets_res = get_assets_res(assets_root_dir).await;
    let k = assets_res
        .iter()
        .map(|(k, _)| k.clone())
        .collect::<Vec<String>>();
    debug!("AAAAAAAA: {:#?}", k);

    let tacker = TaskTracker::new();

    let res_data = ResData {
        leptos_options: leptos_options,
        assets_res: assets_res,
        index_res: index_res,
        not_found_res: not_found_res.to_vec(),
        schemas: schemas,
    };
    let res_data = Arc::new(res_data);

    let mut block_tracker = global::ThresholdTracker::new(time);
    let mut ban_tracker = global::ThresholdTracker::new(time);
    let mut banned_until: global::BanType = None;

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

                tacker.spawn(handle_res(stream, res_data.clone()));
                // let res_data = res_data.clone();
                // tacker.spawn(async move {
                //     stream;
                //     res_data;
                // });
            }
            _ = cancelation_token.cancelled() => {
                break;
            }
        }
    }

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

    tacker.close();
    tacker.wait().await;
}

fn get_assets_res(
    assets_dir: &str,
) -> std::pin::Pin<std::boxed::Box<dyn futures::Future<Output = HashMap<String, Vec<u8>>>>> {
    let assets_dir = assets_dir.to_string();
    std::boxed::Box::pin(async move {
        let mut responses: HashMap<String, Vec<u8>> = HashMap::new();
        debug!("reading {}", assets_dir);
        let mut dir = tokio::fs::read_dir(&assets_dir).await.unwrap();

        while let Some(entry) = dir.next_entry().await.unwrap() {
            let kind = entry.file_type().await.unwrap();
            if kind.is_dir() {
                let name = entry.file_name();
                let name = name.to_str().unwrap();
                let sub_assets_dir = format!("{}/{}", assets_dir, name);
                let sub_responses = get_assets_res(&sub_assets_dir).await;
                for (sub_key, sub_data) in sub_responses {
                    responses.insert(format!("/{}{}", name, sub_key), sub_data);
                }
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
                let new_path = std::path::Path::new(&assets_dir).join(name);
                let new_path = new_path.to_str().unwrap();

                match extension {
                    "ico" => {
                        let bytes = tokio::fs::read(new_path).await.unwrap();
                        let bytes_content_length = bytes.len();
                        let header = format!(
                            "HTTP/1.1 200 OK\r\ncontent-type: x-icon\r\ncontent-length: {bytes_content_length}\r\n\r\n"
                        );
                        let header = header.as_bytes();
                        let res: Vec<u8> = [header, &bytes].concat();
                        let route = format!("/{}", name);
                        responses.insert(route, res);
                    }
                    "webp" => {
                        let bytes = tokio::fs::read(new_path).await.unwrap();
                        let bytes_content_length = bytes.len();
                        let header = format!(
                            "HTTP/1.1 200 OK\r\ncontent-type: image/webp\r\ncontent-length: {bytes_content_length}\r\n\r\n"
                        );
                        let header = header.as_bytes();
                        let res: Vec<u8> = [header, &bytes].concat();
                        let route = format!("/{}", name);
                        responses.insert(route, res);
                    }
                    "svg" => {
                        let bytes = tokio::fs::read(new_path).await.unwrap();
                        let bytes_content_length = bytes.len();
                        let header = format!(
                            "HTTP/1.1 200 OK\r\ncontent-type: image/svg+xml\r\ncontent-length: {bytes_content_length}\r\n\r\n"
                        );
                        let header = header.as_bytes();
                        let res: Vec<u8> = [header, &bytes].concat();
                        let route = format!("/{}", name);
                        responses.insert(route, res);
                    }
                    "css" => {
                        let css = tokio::fs::read(new_path).await.unwrap();
                        let css_content_length = css.len();
                        let css_header = format!(
                            "HTTP/1.1 200 OK\r\ncontent-type: text/css\r\ncontent-length: {css_content_length}\r\n\r\n"
                        );
                        let css_header = css_header.as_bytes();
                        let css_res: Vec<u8> = [css_header, &css].concat();
                        let route = format!("/{}", name);
                        responses.insert(route, css_res);
                    }
                    "js" => {
                        let js: Vec<u8> = tokio::fs::read(new_path).await.unwrap();
                        let js_content_length = js.len();
                        let js_header = format!("HTTP/1.1 200 OK\r\ncontent-type: text/javascript\r\ncontent-length: {js_content_length}\r\n\r\n");
                        let js_header = js_header.as_bytes();
                        let js_res: Vec<u8> = [js_header, &js].concat();
                        let route = format!("/{}", name);
                        responses.insert(route, js_res);
                    }
                    "wasm" => {
                        let wasm = tokio::fs::read(new_path).await.unwrap();
                        let wasm_content_length = wasm.len();
                        let wasm_header = format!("HTTP/1.1 200 OK\r\ncontent-type: application/wasm\r\ncontent-length: {wasm_content_length}\r\n\r\n");
                        let wasm_header = wasm_header.as_bytes();
                        let wasm_res = [wasm_header, &wasm].concat();

                        // let not_found_res = "HTTP/1.1 404 Not Found\r\n\r\n";
                        // let not_found_res = not_found_res.as_bytes().to_vec();

                        let route = format!("/{}", name);
                        responses.insert(route, wasm_res);
                    }
                    _ => {}
                }
            }
        }
        responses
    })
}

async fn handle_res<T: AsRef<str> + std::marker::Send + 'static>(
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

    //let app_fn = (*res_data.app_fn)();
    let result = if let Some(res) = res_data.assets_res.get(path) {
        stream.write_all(res).await
    } else {
        let found = compare_path(path, &res_data.schemas);
        if found {
            cfg_if! {
                if #[cfg(feature = "serve_csr")] {
                    trace!("sending csr app....");
                    stream.write_all(index_res).await
                } else {
                    trace!("rendering app....");

                    let app = render_my_app(&res_data.leptos_options, &full_path).await;
                    stream.write_all(app.as_bytes()).await

                    // stream.write_all(&res_data.not_found_res).await
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
