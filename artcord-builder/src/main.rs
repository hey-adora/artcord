use std::{ffi::OsStr, net::SocketAddr, ops::Deref, path::Path, pin::Pin, process::ExitStatus};

use artcord_leptos_web_sockets::WsPackage;
use artcord_state::global;
use cfg_if::cfg_if;
use dotenv::dotenv;
use futures::{future::join_all, Future, FutureExt, SinkExt, StreamExt, TryStreamExt};
use notify::{
    event::{AccessKind, AccessMode},
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use tokio::join;
use tokio::signal;
use tokio::sync::{broadcast, mpsc, watch};
use tokio::{
    net::TcpListener,
    process::{Child, Command},
};
use tokio::{net::TcpStream, select};
use tokio_tungstenite::tungstenite::{protocol::CloseFrame, Message};
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info, trace, warn, Level};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ProjectKind {
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CompilerState {
    Starting(ProjectKind),
    Compiling(ProjectKind),
    Killing(ProjectKind),
    Ready,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ManagerEventKind {
    File(ProjectKind),
    CompilerStarted(ProjectKind),
    CompilerEnded(ProjectKind),
    CompilerKilled(ProjectKind),
    CompilerFailed(ProjectKind),
    RuntimeReady,
    BrowserReady,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CompilerEventKind {
    Start(ProjectKind),
    Kill,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RuntimeEvent {
    Restart,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum BrowserEvent {
    Restart,
}

// async fn do_stuff_async() {
//     loop {
//         trace!("boop");
//         sleep(Duration::from_secs(1)).await;

//     }
// }

// async fn more_async_work() {
//     trace!("beep");
//     sleep(Duration::from_secs(3)).await;
// }

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .unwrap();
    //tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::builder().parse_lossy("artcord=trace")).try_init().unwrap();
    // cfg_if! {
    //     if #[cfg(feature = "production")] {
    //         tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::builder().parse_lossy("artcord=trace")).try_init().unwrap();
    //     } else {
    //         tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::builder().parse_lossy("artcord=trace")).try_init().unwrap();
    //     }
    // }

    trace!("Started");

    let (send_manager_event, recv_manager_event) = mpsc::channel::<ManagerEventKind>(1000);
    let (send_compiler_event, recv_compiler_event) = broadcast::channel::<CompilerEventKind>(1);
    let (send_runtime_restart_event, recv_runtime_restart_event) =
        broadcast::channel::<RuntimeEvent>(1);
    let (send_exit, recv_exit) = watch::channel::<Option<()>>(None);
    let (send_browser_event, recv_browser_event) = broadcast::channel::<BrowserEvent>(1);
    let mut futs: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();

    let paths_backend = [
        "artcord",
        "artcord-http",
        "artcord-mongodb",
        "artcord-serenity",
        "artcord-state",
        "artcord-tungstenite",
    ];

    let paths_frontend = [
        "artcord-leptos",
        "assets",
        "style",
        "artcord-leptos-web-sockets",
    ];

    for path in paths_backend {
        let fut = watch_dir(
            path,
            send_manager_event.clone(),
            recv_exit.clone(),
            ProjectKind::Back,
        );
        futs.push(fut.boxed());
    }

    for path in paths_frontend {
        let fut = watch_dir(
            path,
            send_manager_event.clone(),
            recv_exit.clone(),
            ProjectKind::Front,
        );
        futs.push(fut.boxed());
    }

    let manager_fut = manager(
        recv_manager_event,
        send_compiler_event.clone(),
        send_runtime_restart_event.clone(),
        send_browser_event,
    );
    futs.push(manager_fut.boxed());

    let compiler_fut = compiler(send_manager_event.clone(), recv_compiler_event);
    futs.push(compiler_fut.boxed());

    let runtime_fut = runtime(recv_runtime_restart_event);
    futs.push(runtime_fut.boxed());

    let socket_fut = sockets(
        recv_exit.clone(),
        recv_browser_event,
        send_manager_event.clone(),
    );
    futs.push(socket_fut.boxed());

    let handle_exit = handle_exit(
        send_exit,
        send_manager_event.clone(),
        send_compiler_event.clone(),
        send_runtime_restart_event.clone(),
    );
    futs.push(handle_exit.boxed());

    join_all(futs).await;
}

async fn handle_exit(
    send_exit: watch::Sender<Option<()>>,
    send_manager_event: mpsc::Sender<ManagerEventKind>,
    send_compiler_event: broadcast::Sender<CompilerEventKind>,
    send_runtime_restart_event: broadcast::Sender<RuntimeEvent>,
) {
    let result = signal::ctrl_c().await;
    if let Err(err) = result {
        error!("handle_exit: err: receiving ctrl+c: {}", err);
        return;
    }
    info!("exiting!");

    let result = send_manager_event.send(ManagerEventKind::Exit).await;
    if let Err(err) = result {
        error!("error sending manager exit signal: {}", err);
    }

    let result = send_compiler_event.send(CompilerEventKind::Exit);
    if let Err(err) = result {
        error!("error sending compiler exit signal: {}", err);
    }

    let result = send_runtime_restart_event.send(RuntimeEvent::Exit);
    if let Err(err) = result {
        error!("error sending runtime exit signal: {}", err);
    }

    let result = send_exit.send(Some(()));
    if let Err(err) = result {
        error!("error sending general exit signal: {}", err);
    }
}

async fn sockets(
    recv_exit: watch::Receiver<Option<()>>,
    recv_browser_event: broadcast::Receiver<BrowserEvent>,
    send_manager_event: mpsc::Sender<ManagerEventKind>,
) {
    let connection_tasks = TaskTracker::new();

    let addr = "0.0.0.0:3001";
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind socket addr");
    info!("socket: restart socket listening on: ws://{}", &addr);

    // while  {

    // }

    let handle_connections = async {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(result) => result,
                Err(err) => {
                    error!("socket: error accepting connection: {}", err);
                    continue;
                }
            };

            let peer = stream.peer_addr().expect("Failed to get peer addr");
            trace!("socket: connected: {}", peer);

            connection_tasks.spawn(sockets_accept_connection(
                peer,
                stream,
                recv_exit.clone(),
                recv_browser_event.resubscribe(),
                send_manager_event.clone(),
            ));
        }
    };

    let handle_exit = {
        let mut recv_exit = recv_exit.clone();
        async move {
            loop {
                let should_exit = { recv_exit.borrow_and_update().deref().is_some() };
                if should_exit {
                    break;
                }
                let result = recv_exit.changed().await;
                if let Err(err) = result {
                    error!("socket: recv: error: {}", err);
                }
            }
            debug!("socket: received exit signal, exiting...");
        }
    };

    select!(
        r = handle_connections => {
            trace!("socket: ws://{} handle_connections exiting...: {:?}", &addr, r);
        },
        _ = handle_exit => {
            trace!("socket: ws://{} handle_exit exiting...", &addr);
        },
    );

    trace!("socket: waiting for connections to exit... {}", &addr);
    connection_tasks.close();
    connection_tasks.wait().await;

    trace!("socket: ws://{} exited", &addr);
}

async fn sockets_accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    recv_exit: watch::Receiver<Option<()>>,
    recv_browser_event: broadcast::Receiver<BrowserEvent>,
    send_manager_event: mpsc::Sender<ManagerEventKind>,
) {
    if let Err(e) = sockets_handle_connection(
        peer,
        stream,
        recv_exit,
        recv_browser_event,
        send_manager_event.clone(),
    )
    .await
    {
        match e {
            tokio_tungstenite::tungstenite::Error::ConnectionClosed
            | tokio_tungstenite::tungstenite::Error::Protocol(_)
            | tokio_tungstenite::tungstenite::Error::Utf8 => (),
            err => error!("socket: Error proccesing connection: {err}"),
        }
    }
}

// enum SocketMsg {
//     Msg(Vec<u8>),
//     Exit
// }

async fn sockets_handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    mut recv_exit: watch::Receiver<Option<()>>,
    mut recv_browser_event: broadcast::Receiver<BrowserEvent>,
    send_manager_event: mpsc::Sender<ManagerEventKind>,
) -> tokio_tungstenite::tungstenite::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("socket: failed to accept connection");
    //trace!("socket: new websocket connection: {}", peer);

    //ws_stream.re
    let (mut write, read) = ws_stream.split();
    let (send_msg, mut recv_msg) = mpsc::channel::<Vec<u8>>(1000);
    // recv_msg.close();
    // send_msg.close();

    //ws_stream.close(None);

    // write.close();
    // loop {
    //     // let handle_msg = async {
    //     //     let result = read.try_next().await;
    //     //     match result {
    //     //         Ok(result) => {
    //     //             if let Some(result) = result {
    //     //                 match result {
    //     //                     tokio_tungstenite::tungstenite::Message::Binary(msg) => {
    //     //                         info!("received binary msg!");
    //     //                     }
    //     //                     _ => {
    //     //                         info!("socket: received uwknown msg");
    //     //                     }
    //     //                 }
    //     //             } else {
    //     //                 trace!("socket_handle: recv: empty msg");
    //     //             }
    //     //         }
    //     //         Err(err) => {
    //     //             error!("socket_handle: recv: error: {}", err);
    //     //         }
    //     //     }
    //     // };

    //     let handle_exit = async {

    //     };

    // }

    let read_fut = {
        // let send_msg = &send_msg;
        read.try_for_each_concurrent(1000, move |msg| {
            //let send_msg = send_msg.clone();
            let send_manager_event = send_manager_event.clone();
            async move {
                match msg {
                    // tokio_tungstenite::tungstenite::Message::Binary(msg) => {

                    // }
                    tokio_tungstenite::tungstenite::Message::Binary(client_msg_bytes) => {
                        let client_msg: Result<WsPackage<global::DebugClientMsg>, _> =
                        global::DebugClientMsg::from_bytes(&client_msg_bytes);

                        let Ok(client_msg) = client_msg.inspect_err(|err| {
                            error!(
                                "socket: error deserializing client msg: {:?} : {}",
                                client_msg_bytes, err
                            );
                        }) else {
                            return Ok(());
                        };
                        trace!("socekt: msg recv: {:?}", &client_msg);
                        let key = client_msg.0;
                        let client_msg = client_msg.1;
                        let send_result = match client_msg {
                            global::DebugClientMsg::BrowserReady => {
                                send_manager_event
                                    .send(ManagerEventKind::BrowserReady)
                                    .await
                            }
                            global::DebugClientMsg::RuntimeReady => {
                                send_manager_event
                                    .send(ManagerEventKind::RuntimeReady)
                                    .await
                            }
                        };

                        if let Err(e) = send_result {
                            error!("socket: sent manager event: error: {}", e);
                        }

                        // let Ok(client_msg) = client_msg else {
                        //     let err = client_msg.err().map(|e|e.to_string()).unwrap_or("unknown error".to_string());
                        //     error!("socket: error deserializing client msg: {:?} : {}", client_msg_bytes, err);
                        //     return Ok(());
                        // };

                        // let client_msg = match client_msg {
                        //     Ok(a) => a,
                        //     Err(err) => {
                        //         error!("socket: error deserializing client msg: {:?} : {}", client_msg_bytes, err);
                        //         return Ok(());
                        //     }
                        // };

                        //send_msg.send();
                    }
                    _ => {
                        warn!("socket: received uwknown msg");
                    }
                }

                Ok(())
            }
        })
    };

    //read.close();

    let write_fut = async move {
        let write_handle = async {
            loop {
                let Some(msg) = recv_msg.recv().await else {
                    error!("socket: recv: closed for {}", peer);
                    return;
                };

                let send_result = write.send(Message::binary(msg)).await;
                if let Err(e) = send_result {
                    error!("socket: sent: error: {}", e);
                }
            }
        };

        let exit_handle = {
            async move {
                loop {
                    let should_exit = { recv_exit.borrow_and_update().deref().is_some() };
                    // trace!(
                    //     "socket_handle_connection({}): received exit value: {}",
                    //     peer,
                    //     should_exit
                    // );
                    if should_exit {
                        trace!("socket_handle_connection({}): exiting... ", peer);
                        break;
                    }
                    let result = recv_exit.changed().await;
                    if let Err(err) = result {
                        error!("socket_handle_connection({}): recv: error: {}", peer, err);
                        break;
                    }
                }
            }
        };

        let close_frame = CloseFrame {
            code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
            reason: std::borrow::Cow::Borrowed("boom"),
        };

        select!(
                _ = write_handle => {

                },
                _ = exit_handle => {
                    let result = write.send(Message::Close(Some(close_frame))).await;
                    if let Err(err) = result {
                        error!("socket_handle_connection({}): send: error: {}", peer, err);
                    }
                },
        );
    };

    let browser_event_handler = {
        async move {
            loop {
                let result = recv_browser_event.recv().await;
                match result {
                    Ok(result) => {
                        trace!("socekt_connection: recv: {:?}", &result);
                        match result {
                            BrowserEvent::Restart => {
                                //trace!("socekt: msg recv: ({},{:?})", key, &client_msg);
                                let restart_package: WsPackage<global::DebugServerMsg> = (
                                    0,
                                    // WsRouteKey::Perm(DebugMsgPermKey::Debug),
                                    global::DebugServerMsg::Restart,
                                );
                                trace!("socekt_connection: send: {:?}", &restart_package);
                                let server_msg = global::DebugServerMsg::as_bytes(&restart_package);

                                match server_msg {
                                    Ok(server_msg) => {
                                        let send_result = send_msg.send(server_msg).await;
                                        if let Err(e) = send_result {
                                            error!("socket_connection: sent: error: {}", e);
                                        }
                                    }
                                    Err(err) => {
                                        error!(
                                            "socket_connection: error serializing server msg: {:?} : {}",
                                            &restart_package, err
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        match err {
                            broadcast::error::RecvError::Closed => {
                                error!("socket_connection({}): recv: error: {}", peer, err);
                                break;
                            }
                            broadcast::error::RecvError::Lagged(missed) => {
                                warn!("socket_connection({}): recv: error: {}", peer, err);
                            }
                        }
                        continue;
                    }
                };
            }
        }
    };

    // let exit_handle = sockets_handle_connection_on_exit(&peer, recv_exit);

    //join!(async { let _ = read_fut.await.inspect_err(|err| error!("socket_connection: error in read: {}", err)); }, async { write.await; });

    //let futures = [read_fut.boxed(), write.boxed()];
    //join_all(futures);
    select!(
            _ = read_fut => {
                //trace!("socket_connection({}): read_fut finished first", peer);
            },
            _ = write_fut => {
                //trace!("socket_connection({}): write_fut finished first", peer);
            },
            _ = browser_event_handler => {

            }

            // _ = exit_handle => {
            //     trace!("socket_connection({}): exit_handle finished first", peer);
            // }
    );

    trace!("socket: disconnected: {}", peer);
    // socket_err.unwrap();
    // tokio_err.unwrap();

    Ok(())
}

// async fn sockets_handle_connection_on_exit(peer: &SocketAddr, mut recv_exit: watch::Receiver<Option<()>>) {
//     loop {

//         let should_exit = recv_exit.borrow_and_update().deref().is_some();
//         if should_exit {
//             break;
//         }
//         let result = recv_exit.changed().await;

//         if let Err(err) = result {
//             error!("socket_handle: error: {}", err);
//         }
//         trace!("socket_handle: exiting ws for {}", peer);
//     }
// }

async fn runtime(mut recv_runtime_restart_event: broadcast::Receiver<RuntimeEvent>) {
    let path_bin = "./target/debug/artcord";
    let path = Path::new(path_bin);

    'main_loop: loop {
        if path.exists() {
            let mut command = Command::new(path_bin);
            let command = command.spawn();
            let Ok(mut command) = command else {
                let err = command
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or("uwknown error".to_string());
                error!("runtime: recv error: {}", err);
                continue;
            };
            trace!("runtime: started");
            select! {
               _ = command.wait() => {
                   trace!("runtime: finished");
                },
                result = recv_runtime_restart_event.recv() => {
                    match result {
                        Ok(result) => {
                            match result {
                                RuntimeEvent::Restart => {
                                    runtime_on_kill(&mut command).await;
                                    continue 'main_loop;
                                }
                                RuntimeEvent::Exit => {
                                    runtime_on_exit(&mut command).await;
                                    break 'main_loop;
                                }
                            }
                        }
                        Err(err) => {
                            error!("runtime: recv error: {}", err);
                        }
                    }
                }
            };
        }

        loop {
            let result = recv_runtime_restart_event.recv().await;
            match result {
                Ok(result) => match result {
                    RuntimeEvent::Restart => {
                        continue 'main_loop;
                    }
                    RuntimeEvent::Exit => {
                        break 'main_loop;
                    }
                },
                Err(err) => {
                    error!("runtime: recv error: {}", err);
                }
            }
        }
    }

    trace!("runtime: exited.");
}
// , recv_exit: watch::Receiver<Option<()>>
async fn runtime_on_exit(command: &mut Child) {
    trace!("runtime: exiting...");
    let command_kill_result = command.kill().await;
    if let Err(err) = command_kill_result {
        error!("Runtime: error killing command: {}", err);
    };
}

async fn runtime_on_kill(command: &mut Child) {
    let command_kill_result = command.kill().await;
    if let Err(err) = command_kill_result {
        error!("Runtime: error killing command: {}", err);
    };
    trace!("runtime: killed, restarting");
}

async fn compiler(
    send_manager_event: mpsc::Sender<ManagerEventKind>,
    mut recv_compiler_event: broadcast::Receiver<CompilerEventKind>,
) {
    let mut commands_backend = build_commands([
        vec![
            "cargo",
            "build",
            "--package",
            "artcord",
            "--features",
            "development,serve_csr,ssr",
        ],
        vec![
            "cargo",
            "build",
            "--package",
            "artcord-leptos",
            "--features",
            "development,csr",
            "--target",
            "wasm32-unknown-unknown",
        ],
        vec!["rm", "-rf", "./target/site"],
        vec!["mkdir", "./target/site"],
        vec!["mkdir", "./target/site/pkg"],
        vec!["cp", "-r", "./assets/.", "./target/site/"],
        vec![
            "tailwindcss",
            "-i",
            "./input.css",
            "-o",
            "./style/output.css",
            "-c",
            "./tailwind.config.js",
        ],
        vec![
            "cp",
            "./style/output.css",
            "./target/site/pkg/leptos_start5.css",
        ],
        vec![
            "wasm-bindgen",
            "./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm",
            "--no-typescript",
            "--target",
            "web",
            "--out-dir",
            "./target/site/pkg",
            "--out-name",
            "leptos_start5",
        ],
    ])
    .await;

    let mut commands_frontend = build_commands([
        vec![
            "cargo",
            "build",
            "--package",
            "artcord-leptos",
            "--features",
            "development,csr",
            "--target",
            "wasm32-unknown-unknown",
        ],
        vec!["rm", "-rf", "./target/site"],
        vec!["mkdir", "./target/site"],
        vec!["mkdir", "./target/site/pkg"],
        vec!["cp", "-r", "./assets/.", "./target/site/"],
        vec![
            "tailwindcss",
            "-i",
            "./input.css",
            "-o",
            "./style/output.css",
            "-c",
            "./tailwind.config.js",
        ],
        vec![
            "cp",
            "./style/output.css",
            "./target/site/pkg/leptos_start5.css",
        ],
        vec![
            "wasm-bindgen",
            "./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm",
            "--no-typescript",
            "--target",
            "web",
            "--out-dir",
            "./target/site/pkg",
            "--out-name",
            "leptos_start5",
        ],
    ])
    .await;

    'main_loop: loop {
        let result = recv_compiler_event.recv().await;
        let Ok(compiler_event_kind) = result else {
            let err = result
                .err()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "uwknown error".to_string());
            error!("Compiler: recv: error: {}", err);
            continue;
        };

        match compiler_event_kind {
            CompilerEventKind::Start(project_kind) => {
                trace!(
                    "Compiler: recv: compiler_event_kind: {:?}",
                    compiler_event_kind
                );

                // match compiler_event_kind {
                //     CompilerEventKind::Kill => {

                //     }
                //     CompilerEventKind::Start(project_kind) => {

                //     }
                // };

                trace!("Compiler: sent: file: CompilerStarted({:?})", project_kind);
                let send_result = send_manager_event
                    .send(ManagerEventKind::CompilerStarted(project_kind))
                    .await;
                if let Err(e) = send_result {
                    error!("Compiler: sent: error: {}", e);
                    continue;
                }

                // sleep(Duration::from_secs(5)).await;

                let commands: &mut [(Command, String)] = match project_kind {
                    ProjectKind::Back => commands_backend.as_mut(),
                    ProjectKind::Front => commands_frontend.as_mut(),
                };

                let commands_count = commands.len();

                for (i, (command, command_name)) in commands.iter_mut().enumerate() {
                    let mut command = command.spawn().unwrap();

                    select! {
                       command_return = command.wait() => {
                           let good = compiler_on_finish(i == commands_count - 1,project_kind, command_return, command_name, send_manager_event.clone()).await;

                           if !good {
                               break;
                           }
                        },
                       exit = compiler_on_run(&mut recv_compiler_event, project_kind, send_manager_event.clone()) => {
                           if exit {
                                compiler_on_exit(&mut command).await;
                                break 'main_loop;
                           } else {
                                compiler_on_kill(&mut command, project_kind, send_manager_event.clone()).await;
                                break;
                           }
                        }
                    };
                }
            }
            CompilerEventKind::Kill => {
                error!(
                    "Compiler: recv: error: received: {:?} before compiler even started.",
                    compiler_event_kind
                );
            }
            CompilerEventKind::Exit => {
                trace!("compiler: exiting...");
                break;
            }
        }
    }

    trace!("compiler: exited.");
}

async fn compiler_on_finish(
    last: bool,
    project_kind: ProjectKind,
    command_return: Result<ExitStatus, std::io::Error>,
    command_name: &str,
    send_manager_event: mpsc::Sender<ManagerEventKind>,
) -> bool {
    let Ok(command_return) = command_return else {
        let err = command_return
            .err()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "uwknown error".to_string());
        error!(
            "Compiler(COMMAND-{:?}): error: compiler_event_kind: {:?}",
            project_kind, err
        );
        return false;
    };

    let good = command_return.success();
    if good {
        trace!(
            "Compiler(COMMAND-{:?}): finished: {}",
            project_kind,
            command_name
        );
        if last {
            trace!(
                "Compiler(COMMAND-{:?}): sent: CompilerEnded({:?})",
                project_kind,
                project_kind
            );
            let send_result = send_manager_event
                .send(ManagerEventKind::CompilerEnded(project_kind))
                .await;
            if let Err(e) = send_result {
                error!("Compiler(RUNNING-{:?}): sent: error: {}", project_kind, e);
            }
        }
        true
    } else {
        error!(
            "Compiler(COMMAND-{:?}): sent: CompilerFailed({:?})",
            project_kind, project_kind
        );
        let send_result = send_manager_event
            .send(ManagerEventKind::CompilerFailed(project_kind))
            .await;
        if let Err(e) = send_result {
            error!("Compiler(RUNNING-{:?}): sent: error: {}", project_kind, e);
        }
        false
    }
}

async fn compiler_on_exit(command: &mut Child) {
    trace!("compiler: exiting...");
    let command_kill_result = command.kill().await;
    if let Err(err) = command_kill_result {
        error!("Compiler: error killing command: {}", err);
    };
}

async fn compiler_on_kill(
    command: &mut Child,
    project_kind: ProjectKind,
    send_manager_event: mpsc::Sender<ManagerEventKind>,
) {
    let command_kill_result = command.kill().await;
    if let Err(err) = command_kill_result {
        error!("Compiler: error killing command: {}", err);
    };

    trace!(
        "Compiler(RUNNING-{:?}): sent: CompilerKilled({:?})",
        project_kind,
        project_kind
    );
    let send_result = send_manager_event
        .send(ManagerEventKind::CompilerKilled(project_kind))
        .await;

    if let Err(e) = send_result {
        error!("Compiler(RUNNING-{:?}): sent: error: {}", project_kind, e);
    }
}

async fn compiler_on_run(
    recv_compiler_event: &mut broadcast::Receiver<CompilerEventKind>,
    project_kind: ProjectKind,
    _send_manager_event: mpsc::Sender<ManagerEventKind>,
) -> bool {
    loop {
        let result = recv_compiler_event.recv().await;
        let Ok(compiler_event_kind) = result else {
            if let Some(err) = result.err() {
                match err {
                    broadcast::error::RecvError::Closed => {
                        error!("compiler_run: recv: closed");
                        return true;
                    }
                    broadcast::error::RecvError::Lagged(msg_count) => {
                        warn!(
                            "compiler_run: recv: lagged, missed msg count: {}",
                            msg_count
                        );
                    }
                }
            } else {
                error!("compiler_run: recv uwknown error");
            }
            continue;
        };

        match compiler_event_kind {
            CompilerEventKind::Kill => {
                trace!(
                    "Compiler(RUNNING-{:?}): recv: CompilerEventKind::Kill",
                    project_kind
                );

                break;
            }
            CompilerEventKind::Start(_project_kind) => {
                error!("Compiler(RUNNING-{:?}): recv: error: received: {:?} while the compiller is already running.", project_kind, compiler_event_kind);
            }
            CompilerEventKind::Exit => {
                return true;
            }
        }
    }

    false
}

async fn manager(
    mut recv_manager_event: mpsc::Receiver<ManagerEventKind>,
    send_compiler_event: broadcast::Sender<CompilerEventKind>,
    send_runtime_restart_event: broadcast::Sender<RuntimeEvent>,
    send_browser_event: broadcast::Sender<BrowserEvent>,
) {
    // let mut back_is_compiling = false;
    // let mut back_is_starting = false;
    // let mut back_is_being_killed = false;
    let mut compiler_state = CompilerState::Ready;
    let mut compile_next: Option<ProjectKind> = None;

    //let _back_is_running = false;

    //let mut browser_needs_restart = false;
    // let mut browser_ready = false;

    let mut event_kind = ManagerEventKind::File(ProjectKind::Back);

    loop {
        match event_kind {
            ManagerEventKind::File(project_kind) => {
                trace!(
                    "Manager: recv: file: {:?}, current compiler state: {:?}",
                    project_kind,
                    compiler_state
                );
                match project_kind {
                    ProjectKind::Back => {
                        match compiler_state {
                            CompilerState::Compiling(current_project_kind) => {
                                //trace!("Manager: skipped: compiler is busy compiling");
                                trace!("Manager: sent: compile: {:?}", CompilerEventKind::Kill);
                                let send_result = send_compiler_event.send(CompilerEventKind::Kill);
                                if let Err(e) = send_result {
                                    error!("Manager: sent compiler event: error: {}", e);
                                    continue;
                                } else {
                                    trace!(
                                        "Manager: set: compiling state: Killing({:?})",
                                        current_project_kind
                                    );
                                    compiler_state = CompilerState::Killing(current_project_kind);
                                    trace!(
                                        "Manager: set: compiling next: Some({:?})",
                                        project_kind
                                    );
                                    compile_next = Some(project_kind);
                                }
                            }
                            CompilerState::Killing(_current_project_kind) => {
                                trace!("Manager: skipped: compiler is busy killing");
                            }
                            CompilerState::Starting(_current_project_kind) => {
                                trace!("Manager: skipped: compiler is busy starting up");
                            }
                            CompilerState::Ready => {
                                if let Some(current_compile_next) = compile_next {
                                    trace!(
                                        "Manager: compile next is set to: Some({:?}), will reset to None",
                                        current_compile_next
                                    );
                                    compile_next = None;
                                }
                                trace!(
                                    "Manager: sent: compile: {:?}",
                                    CompilerEventKind::Start(project_kind)
                                );
                                let send_result = send_compiler_event
                                    .send(CompilerEventKind::Start(project_kind));
                                if let Err(e) = send_result {
                                    error!("Manager: sent compiler event: error: {}", e);
                                    continue;
                                } else {
                                    trace!(
                                        "Manager: set: compiling state: Starting({:?})",
                                        project_kind
                                    );
                                    compiler_state = CompilerState::Starting(project_kind);
                                }
                            }
                        }
                        // if back_is_compiling {
                        //     trace!("Manager: skipped: backend is already compiling");
                        // } else {

                        // }
                    }
                    ProjectKind::Front => match compiler_state {
                        CompilerState::Compiling(current_project_kind) => {
                            trace!("Manager: set: compiling next: Some({:?})", project_kind);
                            if let Some(current_compile_next) = compile_next {
                                trace!(
                                    "Manager: has compiling next already set to Some({:?})",
                                    current_compile_next
                                );
                                match current_compile_next {
                                    ProjectKind::Back => {
                                        //trace!("Manager: skipping: compiling next is already set to Some({:?})", project_kind);
                                        //compile_next = Some(project_kind);

                                        trace!("Manager: sent: {:?}", CompilerEventKind::Kill);
                                        let send_result =
                                            send_compiler_event.send(CompilerEventKind::Kill);
                                        if let Err(e) = send_result {
                                            error!("Manager: sent compiler event: error: {}", e);
                                            continue;
                                        } else {
                                            trace!(
                                                "Manager: set: compiling state: Killing({:?})",
                                                current_project_kind
                                            );
                                            compiler_state =
                                                CompilerState::Killing(current_project_kind);
                                        }
                                    }
                                    ProjectKind::Front => {
                                        match current_project_kind {
                                            ProjectKind::Back => {
                                                trace!("Manager: skipping: compiling next is already set to Some({:?})", project_kind);
                                            }
                                            ProjectKind::Front => {
                                                trace!(
                                                    "Manager: sent: {:?}",
                                                    CompilerEventKind::Kill
                                                );
                                                let send_result = send_compiler_event
                                                    .send(CompilerEventKind::Kill);
                                                if let Err(e) = send_result {
                                                    error!(
                                                        "Manager: sent compiler event: error: {}",
                                                        e
                                                    );
                                                    continue;
                                                } else {
                                                    trace!(
                                                        "Manager: set: compiling state: Killing({:?})",
                                                        current_project_kind
                                                    );
                                                    compiler_state = CompilerState::Killing(
                                                        current_project_kind,
                                                    );
                                                }
                                            }
                                        }

                                        //compile_next = Some(project_kind);
                                    }
                                }
                            } else {
                                compile_next = Some(project_kind);
                            }
                        }
                        CompilerState::Killing(_current_project_kind) => {
                            trace!("Manager: skipped: compiler is busy killing");
                        }
                        CompilerState::Starting(_current_project_kind) => {
                            trace!("Manager: skipped: compiler is busy starting up");
                        }
                        CompilerState::Ready => {
                            if let Some(current_compile_next) = compile_next {
                                trace!(
                                    "Manager: has compiling next already set to Some({:?})",
                                    current_compile_next
                                );

                                trace!(
                                    "Manager: sent: compile: {:?}",
                                    CompilerEventKind::Start(current_compile_next)
                                );
                                let send_result = send_compiler_event
                                    .send(CompilerEventKind::Start(current_compile_next));
                                if let Err(e) = send_result {
                                    error!("Manager: send compiler event: error: {}", e);
                                    continue;
                                } else {
                                    trace!(
                                        "Manager: set: compiling state: Starting({:?})",
                                        current_compile_next
                                    );
                                    compiler_state = CompilerState::Starting(current_compile_next);

                                    trace!("Manager: set: compiling next: None");

                                    compile_next = None;
                                }

                                // match current_compile_next {
                                //     ProjectKind::Back => {

                                //     }
                                //     ProjectKind::Front => {

                                //     }
                                // }
                            } else {
                                trace!(
                                    "Manager: sent: compile: {:?}",
                                    CompilerEventKind::Start(project_kind)
                                );
                                let send_result = send_compiler_event
                                    .send(CompilerEventKind::Start(project_kind));
                                if let Err(e) = send_result {
                                    error!("Manager: send compiler event: error: {}", e);
                                    continue;
                                } else {
                                    trace!(
                                        "Manager: set: compiling state: Starting({:?})",
                                        project_kind
                                    );
                                    compiler_state = CompilerState::Starting(project_kind);
                                }
                            }
                        }
                    },
                }
            }
            ManagerEventKind::CompilerStarted(project_kind) => {
                trace!(
                    "Manager: recv: compilation_started: {:?}, current compiler state: {:?}",
                    project_kind,
                    compiler_state
                );
                match project_kind {
                    ProjectKind::Back => match compiler_state {
                        CompilerState::Compiling(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Starting, not Compiling({:?})", current_project_kind);
                        }
                        CompilerState::Killing(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Starting, not Killing({:?})", current_project_kind);
                        }
                        CompilerState::Starting(current_project_kind) => {
                            if current_project_kind == project_kind {
                                trace!(
                                    "Manager: set: compiler state to Compiling({:?})",
                                    project_kind
                                );
                                compiler_state = CompilerState::Compiling(project_kind);
                            } else {
                                error!("Manager: sync: error: compiler state Starting project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                            }
                        }
                        CompilerState::Ready => {
                            error!("Manager: sync: error: compiler state suppose to be Starting, not ready");
                        }
                    },
                    ProjectKind::Front => match compiler_state {
                        CompilerState::Compiling(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Starting, not Compiling({:?})", current_project_kind);
                        }
                        CompilerState::Killing(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Starting, not Killing({:?})", current_project_kind);
                        }
                        CompilerState::Starting(current_project_kind) => {
                            if current_project_kind == project_kind {
                                trace!(
                                    "Manager: set: compiler state to Compiling({:?})",
                                    project_kind
                                );
                                compiler_state = CompilerState::Compiling(project_kind);
                            } else {
                                error!("Manager: sync: error: compiler state Starting project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                            }
                        }
                        CompilerState::Ready => {
                            error!("Manager: sync: error: compiler state suppose to be Starting, not ready");
                        }
                    },
                }
            }
            ManagerEventKind::CompilerEnded(project_kind) => {
                trace!("Manager: recv: compilation_ended: {:?}", project_kind);
                match project_kind {
                    ProjectKind::Back => match compiler_state {
                        CompilerState::Compiling(current_project_kind) => {
                            if current_project_kind == project_kind {
                                if let Some(next_project_knd) = compile_next {
                                    trace!(
                                        "Manager: sent: compile next: {:?}",
                                        CompilerEventKind::Start(next_project_knd)
                                    );
                                    let send_result = send_compiler_event
                                        .send(CompilerEventKind::Start(next_project_knd));
                                    if let Err(e) = send_result {
                                        error!("Manager: sent: error: {}", e);
                                        continue;
                                    } else {
                                        trace!(
                                            "Manager: set: compiling state: Starting({:?})",
                                            next_project_knd
                                        );
                                        compiler_state = CompilerState::Starting(next_project_knd);
                                        trace!("Manager: set: compiling next: None");
                                        compile_next = None;
                                    }
                                } else {
                                    trace!("Manager: set: compiler state to Ready");
                                    compiler_state = CompilerState::Ready;
                                    let send_result =
                                        send_runtime_restart_event.send(RuntimeEvent::Restart);
                                    if let Err(e) = send_result {
                                        error!("Manager: sent runtime restart event: error: {}", e);
                                        continue;
                                    }
                                }
                            } else {
                                error!("Manager: sync: error: compiler state Compiling project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                            }
                        }
                        CompilerState::Killing(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not Killing({:?})", current_project_kind);
                        }
                        CompilerState::Starting(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not Starting({:?})", current_project_kind);
                        }
                        CompilerState::Ready => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not ready");
                        }
                    },
                    ProjectKind::Front => match compiler_state {
                        CompilerState::Compiling(current_project_kind) => {
                            if current_project_kind == project_kind {
                                if let Some(next_project_knd) = compile_next {
                                    trace!(
                                        "Manager: sent: compile next: {:?}",
                                        CompilerEventKind::Start(next_project_knd)
                                    );
                                    let send_result = send_compiler_event
                                        .send(CompilerEventKind::Start(next_project_knd));
                                    if let Err(e) = send_result {
                                        error!("Manager: sent: error: {}", e);
                                        continue;
                                    } else {
                                        trace!(
                                            "Manager: set: compiling state: Starting({:?})",
                                            next_project_knd
                                        );
                                        compiler_state = CompilerState::Starting(next_project_knd);
                                        trace!("Manager: set: compiling next: None");
                                        compile_next = None;
                                    }
                                } else {
                                    trace!("Manager: set: compiler state to Ready");
                                    compiler_state = CompilerState::Ready;
                                    let send_result =
                                        send_browser_event.send(BrowserEvent::Restart);
                                    // let send_result = send_runtime_restart_event.send(RuntimeEvent::Restart);
                                    if let Err(e) = send_result {
                                        error!("Manager: sent browser restart event: error: {}", e);
                                        continue;
                                    }
                                }

                                // trace!("Manager: set: compiler state to Ready");
                                // compiler_state = CompilerState::Ready;
                                // todo: restart frontend
                            } else {
                                error!("Manager: sync: error: compiler state Compiling project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                            }
                        }
                        CompilerState::Killing(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not Killing({:?})", current_project_kind);
                        }
                        CompilerState::Starting(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not Starting({:?})", current_project_kind);
                        }
                        CompilerState::Ready => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not ready");
                        }
                    },
                }
            }
            ManagerEventKind::CompilerKilled(project_kind) => {
                trace!("Manager: recv: compilation_ended: {:?}", project_kind);
                match project_kind {
                    ProjectKind::Back => match compiler_state {
                        CompilerState::Compiling(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Killing({:?}), not Compiling({:?})", project_kind, current_project_kind);
                        }
                        CompilerState::Killing(current_project_kind) => {
                            if current_project_kind == project_kind {
                                if let Some(next_project_knd) = compile_next {
                                    if let (ProjectKind::Back, ProjectKind::Front) =
                                        (current_project_kind, next_project_knd)
                                    {
                                        error!("Manager: error: killing backend to compile frontend, something is wrong.");
                                    }

                                    trace!(
                                        "Manager: sent: compile next: {:?}",
                                        CompilerEventKind::Start(next_project_knd)
                                    );
                                    let send_result = send_compiler_event
                                        .send(CompilerEventKind::Start(next_project_knd));
                                    if let Err(e) = send_result {
                                        error!("Manager: sent: error: {}", e);
                                        continue;
                                    } else {
                                        trace!(
                                            "Manager: set: compiling state: Starting({:?})",
                                            next_project_knd
                                        );
                                        compiler_state = CompilerState::Starting(next_project_knd);
                                        trace!("Manager: set: compiling next: None");
                                        compile_next = None;
                                    }
                                } else {
                                    error!("Manager: error: killed{:?} with no compile_next set, something is wrong", current_project_kind);
                                    compiler_state = CompilerState::Ready;
                                }
                            } else {
                                error!("Manager: sync: error: compiler state Killing project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                            }
                        }
                        CompilerState::Starting(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Killing({:?}), not Starting({:?})", project_kind, current_project_kind);
                        }
                        CompilerState::Ready => {
                            error!("Manager: sync: error: compiler state suppose to be Killing({:?}), not ready", project_kind);
                        }
                    },
                    ProjectKind::Front => match compiler_state {
                        CompilerState::Compiling(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Killing({:?}), not Compiling({:?})", project_kind, current_project_kind);
                        }
                        CompilerState::Killing(current_project_kind) => {
                            if current_project_kind == project_kind {
                                if let Some(next_project_knd) = compile_next {
                                    trace!(
                                        "Manager: sent: compile next: {:?}",
                                        CompilerEventKind::Start(next_project_knd)
                                    );
                                    let send_result = send_compiler_event
                                        .send(CompilerEventKind::Start(next_project_knd));
                                    if let Err(e) = send_result {
                                        error!("Manager: sent: error: {}", e);
                                        continue;
                                    } else {
                                        trace!(
                                            "Manager: set: compiling state: Starting({:?})",
                                            next_project_knd
                                        );
                                        compiler_state = CompilerState::Starting(next_project_knd);
                                        trace!("Manager: set: compiling next: None");
                                        compile_next = None;
                                    }
                                } else {
                                    error!("Manager: error: killed{:?} with no compile_next set, something is wrong", current_project_kind);
                                    compiler_state = CompilerState::Ready;
                                }
                            } else {
                                error!("Manager: sync: error: compiler state Killing project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                            }
                        }
                        CompilerState::Starting(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Killing({:?}), not Starting({:?})", project_kind, current_project_kind);
                        }
                        CompilerState::Ready => {
                            error!("Manager: sync: error: compiler state suppose to be Killing({:?}), not ready", project_kind);
                        }
                    },
                }
            }
            ManagerEventKind::CompilerFailed(project_kind) => {
                trace!("Manager: recv: CompilerFailed: {:?}", project_kind);
                match project_kind {
                    ProjectKind::Back => match compiler_state {
                        CompilerState::Compiling(current_project_kind) => {
                            if current_project_kind == project_kind {
                                trace!("Manager: set: compiler state to Ready");
                                compiler_state = CompilerState::Ready;
                                trace!("Manager: set: compile next to Some({:?})", project_kind);
                                compile_next = Some(project_kind);
                            } else {
                                error!("Manager: sync: error: compiler state Compiling project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                            }
                        }
                        CompilerState::Killing(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not Killing({:?})", current_project_kind);
                        }
                        CompilerState::Starting(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not Starting({:?})", current_project_kind);
                        }
                        CompilerState::Ready => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not ready");
                        }
                    },
                    ProjectKind::Front => match compiler_state {
                        CompilerState::Compiling(current_project_kind) => {
                            if current_project_kind == project_kind {
                                trace!("Manager: set: compiler state to Ready");
                                compiler_state = CompilerState::Ready;
                            } else {
                                error!("Manager: sync: error: compiler state Compiling project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                            }
                        }
                        CompilerState::Killing(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not Killing({:?})", current_project_kind);
                        }
                        CompilerState::Starting(current_project_kind) => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not Starting({:?})", current_project_kind);
                        }
                        CompilerState::Ready => {
                            error!("Manager: sync: error: compiler state suppose to be Compiling, not ready");
                        }
                    },
                }
            }
            ManagerEventKind::Exit => {
                trace!("manager: exiting...");
                break;
            }
            ManagerEventKind::BrowserReady => {
                trace!("Manager: recv BrowserReady");
                // if browser_needs_restart {
                //     // trace!("Manager: set: compiler state to Ready");
                //     // compiler_state = CompilerState::Ready;

                // }
            }
            ManagerEventKind::RuntimeReady => {
                trace!("Manager: sending browser restart event");
                let send_result = send_browser_event.send(BrowserEvent::Restart);
                // let send_result = send_runtime_restart_event.send(RuntimeEvent::Restart);
                if let Err(e) = send_result {
                    error!("Manager: sent browser restart event: error: {}", e);
                    continue;
                }
            }
        }

        if let Some(new_event_kind) = recv_manager_event.recv().await {
            event_kind = new_event_kind;
        }
    }
    trace!("manager: exited.");
}

async fn watch_dir(
    path: &str,
    send_manager_event: mpsc::Sender<ManagerEventKind>,
    mut recv_exit: watch::Receiver<Option<()>>,
    event_kind: ProjectKind,
) {
    trace!("Watcher: watching: {}", path);

    let (tx, mut rx) = mpsc::channel(1);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )
    .unwrap();

    watcher
        .watch(path.as_ref(), RecursiveMode::Recursive)
        .unwrap();

    let watch = async {
        loop {
            let Some(result) = rx.recv().await else {
                error!("watcher({}): closed", path);
                break;
            };

            let Ok(event) = result else {
                let err = result
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "uwknown error".to_string());
                error!("watcher: recv: error: {}", err);

                continue;
            };

            if let EventKind::Access(AccessKind::Close(AccessMode::Write)) = event.kind {
                trace!("Wachter({}): sent: {:?}", path, event_kind);
                let send_result = send_manager_event
                    .send(ManagerEventKind::File(event_kind))
                    .await;
                if let Err(e) = send_result {
                    error!("Watcher: sent manager event: error: {}", e);
                }
            }
        }
    };

    let exit = async {
        loop {
            let should_exit = { recv_exit.borrow_and_update().deref().is_some() };
            if should_exit {
                break;
            }
            let result = recv_exit.changed().await;
            if let Err(err) = result {
                error!("watcher: error: {}", err);
            }
            trace!("watch({}): exiting...", path);
        }
    };

    select! {
        _ = watch => {

        },
        _ = exit => {

        }
    }

    trace!("watch({}): exited.", path);
}

async fn build_commands<I>(commands_parts: I) -> Vec<(Command, String)>
where
    I: IntoIterator,
    I::Item: IntoIterator,
    <I::Item as IntoIterator>::Item: AsRef<OsStr> + AsRef<str>,
{
    let mut commands: Vec<(Command, String)> = Vec::new();

    for command_parts in commands_parts {
        let mut command: Option<Command> = None;
        let mut command_str: String = String::new();
        for part in command_parts {
            command_str.push(' ');
            match command.as_mut() {
                Some(command) => {
                    command_str.push_str(part.as_ref());
                    command.arg(part);
                }
                None => {
                    command_str.push_str(part.as_ref());
                    command = Some(Command::new(part));
                }
            }
        }
        if let Some(command) = command {
            commands.push((command, command_str));
        }
    }

    commands
}
