use std::{ffi::OsStr, net::SocketAddr, path::Path, pin::Pin, process::ExitStatus};

use cfg_if::cfg_if;
use futures::{future::join_all, Future, FutureExt, SinkExt, StreamExt, TryStreamExt};
use notify::{
    event::{AccessKind, AccessMode},
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use tokio::{
    sync::{broadcast, mpsc},
};
use tokio::{join};
use tokio::{
    net::TcpListener,
    process::{Child, Command},
};
use tokio::{net::TcpStream, select};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, trace, Level};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ProjectKind {
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ManagerEventKind {
    File(ProjectKind),
    CompilerStarted(ProjectKind),
    CompilerEnded(ProjectKind),
    CompilerKilled(ProjectKind),
    CompilerFailed(ProjectKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CompilerState {
    Starting(ProjectKind),
    Compiling(ProjectKind),
    Killing(ProjectKind),
    Ready,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CompilerEventKind {
    Start(ProjectKind),
    Kill,
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
    cfg_if! {
        if #[cfg(feature = "production")] {
            tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init().unwrap();
        } else {
            tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init().unwrap();
        }
    }

    trace!("Started");

    let (send_manager_event, recv_manager_event) = mpsc::channel::<ManagerEventKind>(1000);
    let (send_compiler_event, recv_compiler_event) = broadcast::channel::<CompilerEventKind>(1);
    let (send_runtime_restart_event, recv_runtime_restart_event) = broadcast::channel::<()>(1);
    let mut futs: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();

    let paths_backend = [
        "artcord",
        "artcord-actix",
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
        let fut = watch_dir(path, send_manager_event.clone(), ProjectKind::Back);
        futs.push(fut.boxed());
    }

    for path in paths_frontend {
        let fut = watch_dir(path, send_manager_event.clone(), ProjectKind::Front);
        futs.push(fut.boxed());
    }

    let manager_fut = manager(
        recv_manager_event,
        send_compiler_event,
        send_runtime_restart_event,
    );
    futs.push(manager_fut.boxed());

    let compiler_fut = compiler(send_manager_event.clone(), recv_compiler_event);
    futs.push(compiler_fut.boxed());

    let runtime_fut = runtime(recv_runtime_restart_event);
    futs.push(runtime_fut.boxed());

    let socket_fut = sockets();
    futs.push(socket_fut.boxed());

    // send_manager_event
    //     .send(ManagerEventKind::File(ProjectKind::Back))
    //     .await
    //     .expect("Failed to send initial msg.");

    // tokio::select! {
    //     _ = do_stuff_async() => {
    //         trace!("do_stuff_async() completed first")
    //     }
    //     _ = more_async_work() => {
    //         trace!("more_async_work() completed first")
    //     }
    // };

    join_all(futs).await;
}

async fn sockets() {
    let addr = "0.0.0.0:3001";
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind socket addr");
    info!("socket: restart socket listening on: ws://{}", &addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("Failed to get peer addr");
        info!("socket: connected: {}", peer);

        tokio::spawn(sockets_accept_connection(peer, stream));
        // unimplemented!();
    }

}

async fn sockets_accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = sockets_handle_connection(peer, stream).await {
        match e {
            tokio_tungstenite::tungstenite::Error::ConnectionClosed
            | tokio_tungstenite::tungstenite::Error::Protocol(_)
            | tokio_tungstenite::tungstenite::Error::Utf8 => (),
            err => error!("socket: Error proccesing connection: {err}"),
        }
    }
}



async fn sockets_handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
) -> tokio_tungstenite::tungstenite::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("socket: failed to accept connection");
    info!("socket: new websocket connection: {}", peer);
    let (mut write, read) = ws_stream.split();
    let (send_msg, mut recv_msg) = mpsc::channel::<String>(1000);
    

    let read = read.try_for_each_concurrent(1000, move |msg| {
        let send_msg = send_msg.clone();
        async move { 
            match msg {
                // tokio_tungstenite::tungstenite::Message::Binary(msg) => {
    
                // }
                tokio_tungstenite::tungstenite::Message::Text(msg) => {
                    debug!("socekt: msg recv: {}", msg);
                    let send_result = send_msg.send(String::from("restart")).await;
                    if let Err(e) = send_result {
                        error!("socket: sent: error: {}", e);
                    }
                }
                _ => {
                    info!("socket: received uwknown msg");
                }
            }
    
            Ok(())
        }
    });

    let write = async move {
        while let Some(msg) = recv_msg.recv().await {
            let send_result = write.send(Message::Text(msg)).await;
            if let Err(e) = send_result {
                error!("socket: sent: error: {}", e);
            }
        }
    };


    let r = join!(read, write);
    
    r.0.unwrap();

    info!("socket: disconnected: {}", peer);
    // socket_err.unwrap();
    // tokio_err.unwrap();

    Ok(())
}

async fn runtime(mut recv_runtime_restart_event: broadcast::Receiver<()>) {
    let path_bin = "./target/debug/artcord";
    let path = Path::new(path_bin);

    loop {
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
            info!("runtime: started");
            select! {
               _ = command.wait() => {
                   info!("runtime: exited");
                },
               _ = recv_runtime_restart_event.recv() => {
                   runtime_on_kill(&mut command).await;
                   continue;
                }
            };
        }

        let result = recv_runtime_restart_event.recv().await;
        if let Err(e) = result {
            error!("runtime: recv error: {}", e);
        }
    }
}

async fn runtime_on_kill(command: &mut Child) {
    let command_kill_result = command.kill().await;
    if let Err(err) = command_kill_result {
        error!("Runtime: error killing command: {}", err);
    };
    info!("runtime: killed, restarting");
}

async fn compiler(
    send_manager_event: mpsc::Sender<ManagerEventKind>,
    mut recv_compiler_event: broadcast::Receiver<CompilerEventKind>,
) {
    let mut commands_backend = build_commands([
        vec!["cargo", "--frozen", "build", "--package", "artcord"],
        vec![
            "cargo",
            "build",
            "--frozen",
            "--package",
            "artcord-leptos",
            "--features",
            "csr",
            "--target",
            "wasm32-unknown-unknown",
        ],
        vec!["rm", "-r", "./target/site"],
        vec!["mkdir", "./target/site"],
        vec!["mkdir", "./target/site/pkg"],
        vec!["cp", "-r", "./assets/.", "./target/site/"],
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
            "--frozen",
            "--package",
            "artcord-leptos",
            "--features",
            "csr",
            "--target",
            "wasm32-unknown-unknown",
        ],
        vec!["rm", "-r", "./target/site"],
        vec!["mkdir", "./target/site"],
        vec!["mkdir", "./target/site/pkg"],
        vec!["cp", "-r", "./assets/.", "./target/site/"],
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

    loop {
        let result = recv_compiler_event.recv().await;
        let Ok(compiler_event_kind) = result else {
            let err = result
                .err()
                .and_then(|e| Some(e.to_string()))
                .unwrap_or_else(|| "uwknown error".to_string());
            error!("Compiler: recv: error: {}", err);
            continue;
        };
        let CompilerEventKind::Start(project_kind) = compiler_event_kind else {
            error!(
                "Compiler: recv: error: received: {:?} before compiler even started.",
                compiler_event_kind
            );
            continue;
        };
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
               _ = compiler_on_run(&mut recv_compiler_event, project_kind, send_manager_event.clone()) => {
                   compiler_on_kill(&mut command, project_kind, send_manager_event.clone()).await;
                   break;
                }
            };
        }
    }
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
            .and_then(|e| Some(e.to_string()))
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
        trace!(
            "Compiler(COMMAND-{:?}): sent: CompilerFailed({:?})",
            project_kind,
            project_kind
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
) {
    while let Ok(compiler_event_kind) = recv_compiler_event.recv().await {
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
        }
    }
}

async fn manager(
    mut recv_manager_event: mpsc::Receiver<ManagerEventKind>,
    send_compiler_event: broadcast::Sender<CompilerEventKind>,
    send_runtime_restart_event: broadcast::Sender<()>,
) {
    // let mut back_is_compiling = false;
    // let mut back_is_starting = false;
    // let mut back_is_being_killed = false;
    let mut compiler_state = CompilerState::Ready;
    let mut compile_next: Option<ProjectKind> = None;

    let _back_is_running = false;

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
                                    compiler_state = CompilerState::Killing(project_kind);
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
                        CompilerState::Compiling(_current_project_kind) => {
                            trace!("Manager: set: compiling next: Some({:?})", project_kind);
                            compile_next = Some(project_kind);
                        }
                        CompilerState::Killing(_current_project_kind) => {
                            trace!("Manager: skipped: compiler is busy killing");
                        }
                        CompilerState::Starting(_current_project_kind) => {
                            trace!("Manager: skipped: compiler is busy starting up");
                        }
                        CompilerState::Ready => {
                            trace!(
                                "Manager: sent: compile: {:?}",
                                CompilerEventKind::Start(project_kind)
                            );
                            let send_result =
                                send_compiler_event.send(CompilerEventKind::Start(project_kind));
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
                                    let send_result = send_runtime_restart_event.send(());
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
                                trace!("Manager: set: compiler state to Ready");
                                compiler_state = CompilerState::Ready;
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
        }

        if let Some(new_event_kind) = recv_manager_event.recv().await {
            event_kind = new_event_kind;
        }
    }
}

async fn watch_dir(
    path: &str,
    send_manager_event: mpsc::Sender<ManagerEventKind>,
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

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if let EventKind::Access(kind) = event.kind {
                    if let AccessKind::Close(kind) = kind {
                        if let AccessMode::Write = kind {
                            trace!("Wachter({}): sent: {:?}", path, event_kind);
                            let send_result = send_manager_event
                                .send(ManagerEventKind::File(event_kind))
                                .await;
                            if let Err(e) = send_result {
                                error!("Watcher: sent manager event: error: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }
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
            match command.as_mut() {
                Some(command) => {
                    command_str.push_str(part.as_ref());
                    command.arg(part);
                }
                None => {
                    command_str.push_str(part.as_ref());
                    command_str.push(' ');
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
