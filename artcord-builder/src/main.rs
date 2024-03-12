use std::ffi::OsStr;

use std::pin::Pin;
use std::process::ExitStatus;

use std::sync::Arc;

use chrono::Utc;
use futures::future::join_all;
use futures::{Future, FutureExt};
use notify::event::{AccessKind, AccessMode};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use tokio::process::Child;
use tokio::process::Command;
use tokio::select;
use tokio::sync::mpsc;

use tokio::sync::broadcast;
use tokio::sync::RwLock;

const SMOOTHING_TOLERENCE: i64 = 100;

#[derive(Clone, Debug)]
enum SignalKind {
    Trigger,
    TriggerTwice,
    TriggerAndCompileOther,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CompKind {
    Front,
    Back,
}

#[derive(Clone, Debug)]
enum CompilationKind {
    Front,
    Back,
    BackPlusFront(broadcast::Sender<(CompilationKind, SignalKind)>),
}

#[tokio::main]
async fn main() {
    let compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>> = Arc::new(RwLock::new(None));
    let (send_front, recv_front) = broadcast::channel::<(CompilationKind, SignalKind)>(1);
    let (send_back, recv_back) = broadcast::channel::<(CompilationKind, SignalKind)>(1);

    let mut commands_backend = build_commands([
        vec!["cargo", "build", "--package", "artcord-leptos"],
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
        vec!["cargo", "build", "--package", "artcord"],
        vec!["./target/debug/artcord"],
    ])
    .await;

    let mut commands_frontend = build_commands([
        vec!["cargo", "build", "--package", "artcord-leptos"],
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

    let mut futs: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();

    let paths_backend = [
        "artcord",
        "artcord-actix",
        "artcord-mongodb",
        "artcord-serenity",
        "artcord-state",
        "artcord-tungstenite",
    ];
    let paths_frontend = ["artcord-leptos", "assets"];

    for path in paths_backend {
        let fut = watch_dir(
            path,
            watch_dir_back_callback,
            send_back.clone(),
            send_front.clone(),
            compiling_state.clone(),
        );
        futs.push(fut.boxed());
    }

    for path in paths_frontend {
        let fut = watch_dir(
            path,
            watch_dir_front_callback,
            send_back.clone(),
            send_front.clone(),
            compiling_state.clone(),
        );
        futs.push(fut.boxed());
    }

    let handler_frontend = proccess(
        (send_front.clone(), recv_front),
        compiling_state.clone(),
        false,
        "FE",
        &mut commands_frontend,
    )
    .boxed();
    futs.push(handler_frontend);

    let handler_backend = proccess(
        (send_back.clone(), recv_back),
        compiling_state.clone(),
        true,
        "BE",
        &mut commands_backend,
    )
    .boxed();
    futs.push(handler_backend);

    join_all(futs).await;
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
            command_str.push_str(part.as_ref());
            match command.as_mut() {
                Some(command) => {
                    command.arg(part);
                }
                None => {
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

// async fn compile(mut recv_comps: broadcast::Receiver<CompKind>) {
//     let mut comps: [Option<CompKind>; 2] = [None; 2];
//     loop {
//         for comp in comps.iter_mut() {
//             *comp = None;
//         }
//         let mut received_comp = recv_comps.recv().await;
//         match received_comp {
//             Ok(received_comp) => {
//                 let kill: bool = match received_comp {
//                     CompKind::Back => {
//                         match &comps[..] {
//                             &[None, None] | 
//                             &[Some(CompKind::Front), None] => {
//                                 comps[0] = Some(CompKind::Back);
//                             }
//                             &[None, Some(_)] => {
//                                 comps[0] = Some(CompKind::Back);
//                                 comps[1] = None;
//                             }
//                             &[Some(CompKind::Back), Some(_)] => {
//                                 comps[1] = None;
//                             }
//                             &[Some(CompKind::Back), None] => {
//                                 // do nothing
//                             }
//                             _ => {
//                                 comps[0] = Some(CompKind::Back);
//                                 comps[1] = None;
//                                 println!("Missed pattern for backend: {:?}", &comps);
//                             }
//                         };
//                         true
//                     }
//                     CompKind::Front => {
//                         match &comps[..] {
//                             &[None, None] => {
//                                 comps[0] = Some(CompKind::Front);
//                                 true
//                             }
//                             &[None, Some(_)] => {
//                                 comps[1] = None;
//                                 true
//                             }
//                             &[Some(CompKind::Back), Some(CompKind::Front)] => {
//                                 false
//                             }
//                             &[Some(CompKind::Front), Some(_)] => {
//                                 comps[1] = None;
//                                 true
//                             }
//                             _ => {
//                                 comps[0] = Some(CompKind::Front);
//                                 comps[1] = None;
//                                 println!("Missed pattern for frontend: {:?}", &comps);
//                                 true
//                             }
//                         }
//                     }
//                 }
//             }
//             Err(e) => {
//                 println!("Recv error: {}", e);
//             }
//         }
//     }
// }

async fn proccess(
    channel: (
        broadcast::Sender<(CompilationKind, SignalKind)>,
        broadcast::Receiver<(CompilationKind, SignalKind)>,
    ),
    compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>,
    inifite: bool,
    name: &str,
    commands: &mut [(Command, String)],
) {
    let (send, mut recv) = channel;
    let command_count = commands.len();

    loop {
        let result = recv.recv().await;
        let Ok((compilation_kind, signal_kind)) = result else {
            println!(
                "{} recv error: {}",
                name,
                result
                    .err()
                    .and_then(|e| Some(e.to_string()))
                    .unwrap_or_else(|| "uwknown error".to_string())
            );
            return;
        };
        println!(
            "received:, CURRENT STATE: {:?}",
            (*compiling_state.read().await)
        );
        println!(
            "{} received: {:?} {:?}",
            name, &compilation_kind, &signal_kind
        );
        {
            let compiling_state = &mut *compiling_state.write().await;
            *compiling_state = Some((Utc::now().timestamp_millis(), compilation_kind));
            if let Some((_, CompilationKind::BackPlusFront(_))) = compiling_state {
                println!("{} skipping", name);
                continue;
            }
        }

        'command_loop: for (i, (command, command_name)) in commands.iter_mut().enumerate() {
            let mut command = command.spawn().unwrap();

            select! {
               command_return = command.wait() => {
                   let good = proccess_on_finish(i, command_return, command, command_name, inifite, command_count, compiling_state.clone(), name, signal_kind.clone()).await;

                   if !good {
                       break 'command_loop;
                   }
                },
                received_value = recv.recv() => {
                   proccess_on_trigger(i, received_value, command, send.clone(), command_name, compiling_state.clone(), name).await;
                   break 'command_loop;
                }
            };
        }
        // println!(
        //     "END LOOP:, CURRENT STATE: {:?}",
        //     (*compiling_state.read().await)
        // );
        // (*compiling_state.write().await) = None;
        // println!(
        //     "END LOOP SET STATE:, CURRENT STATE: {:?}",
        //     (*compiling_state.read().await)
        // );
    }
}

async fn proccess_on_trigger(
    _i: usize,
    received_signal: Result<(CompilationKind, SignalKind), broadcast::error::RecvError>,
    mut command: Child,
    send: broadcast::Sender<(CompilationKind, SignalKind)>,
    command_name: &str,
    compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>,
    name: &str,
) {
    println!("{} Killed: {}", name, command_name);

    command.kill().await.unwrap();

    let prev_compilation_state = {
        let compilation_state = &mut *compiling_state.write().await;
        let output = compilation_state.clone();
        *compilation_state = None;
        output
    };

    

    let Ok((compilation_kind, signal_kind)) = received_signal else {
        println!(
            "{} recv error: {}",
            name,
            received_signal
                .err()
                .and_then(|e| Some(e.to_string()))
                .unwrap_or_else(|| "uwknown error".to_string())
        );
        return;
    };

    match signal_kind {
        SignalKind::TriggerAndCompileOther => {
            if let Some((_, CompilationKind::BackPlusFront(send))) = prev_compilation_state {
                send.send((compilation_kind, SignalKind::TriggerTwice))
                .unwrap();
            }
        }
        SignalKind::TriggerTwice => {
            send.send((compilation_kind, SignalKind::Trigger)).unwrap();
        }
        SignalKind::Trigger => {}
    }
}

async fn proccess_on_finish(
    i: usize,
    command_return: Result<ExitStatus, std::io::Error>,
    _command: Child,
    command_name: &str,
    infinite: bool,
    command_count: usize,
    compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>,
    name: &str,
    signal_kind: SignalKind,
) -> bool {
    {
        if i == command_count
            .checked_sub(if infinite { 2 } else { 1 })
            .unwrap_or(0)
        {
            // let send = {
            //     let mut compiling_state = &mut *compiling_state.write().await;
            //     let send: Option<(broadcast::Sender<(CompilationKind, SignalKind)>, SignalKind)> =
            //         if let Some((i, compilation_kind)) = compiling_state {
            //             if let CompilationKind::BackPlusFront(send) = compilation_kind {
            //                 if let SignalKind::TriggerAndCompileOther = signal_kind.clone() {
            //                     Some((send.clone(), signal_kind))
            //                 } else {
            //                     println!(
            //                         "{} Error, bad state, signal kind should be BackAndFront, is: {:?}",
            //                         name, signal_kind.clone()
            //                     );
            //                     None
            //                 }
            //             } else {
            //                 None
            //             }
            //         } else {
            //             None
            //         };

            //     println!("{} state nulled", name);

            //     *compiling_state = None;

            //     send
            // };

            let send = {
                let mut compiling_state = &mut *compiling_state.write().await;
                let send: Option<broadcast::Sender<(CompilationKind, SignalKind)>> =
                    if let Some((i, compilation_kind)) = compiling_state {
                        if let CompilationKind::BackPlusFront(send) = compilation_kind {
                            Some(send.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                println!("{} state nulled", name);

                *compiling_state = None;

                send
            };

            if let Some(send) = send {
                println!("{} FrontAndBackend signal sent", name);
                let r = send.send((CompilationKind::Front, SignalKind::Trigger));
                if let Err(e) = r {
                    println!("{} Error sending: {}", name, e);
                }
            }
        }
    }

    let Ok(command_return) = command_return else {
        println!(
            "{} recv error: {}",
            name,
            command_return
                .err()
                .and_then(|e| Some(e.to_string()))
                .unwrap_or_else(|| "uwknown error".to_string())
        );
        return false;
    };

    let good = command_return.success();
    if good {
        println!("{} Finished: {}", name, command_name);
        true
    } else {
        println!(
            "{} Error[{}]: {}, ",
            name,
            command_return.code().unwrap_or(0),
            command_name
        );
        false
    }
}

async fn watch_dir_back_callback(
    compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>,
    send_back: broadcast::Sender<(CompilationKind, SignalKind)>,
    send_front: broadcast::Sender<(CompilationKind, SignalKind)>,
) {
    let compiling_state_ref = &*compiling_state.read().await;
    let Some(compiling_state_copy) = compiling_state_ref else {
        send_back
            .send((CompilationKind::Back, SignalKind::Trigger))
            .unwrap();
        return;
    };

    let current_time = Utc::now().timestamp_millis();
    let (past_time, compilation_kind) = compiling_state_copy;

    let time_passed = current_time - past_time;
    if time_passed > SMOOTHING_TOLERENCE {
        match compilation_kind {
            CompilationKind::Back => {
                send_back
                    .send((CompilationKind::Back, SignalKind::TriggerTwice))
                    .unwrap();
            }
            CompilationKind::Front => {
                send_front
                    .send((
                        CompilationKind::Back,
                        SignalKind::TriggerAndCompileOther,
                    ))
                    .unwrap();
            }
            CompilationKind::BackPlusFront(_) => {
                send_back
                    .send((CompilationKind::Back, SignalKind::TriggerTwice))
                    .unwrap();
            }
        }
    }
}

async fn watch_dir_front_callback(
    compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>,
    send_back: broadcast::Sender<(CompilationKind, SignalKind)>,
    send_front: broadcast::Sender<(CompilationKind, SignalKind)>,
) {
    let compiling_state_ref = &*compiling_state.read().await;
    let Some(compiling_state_copy) = compiling_state_ref else {
        send_front
            .send((CompilationKind::Front, SignalKind::Trigger))
            .unwrap();
        return;
    };

    let current_time = Utc::now().timestamp_millis();
    let (past_time, compilation_kind) = compiling_state_copy;

    let time_passed = current_time - past_time;
    if time_passed > SMOOTHING_TOLERENCE {
        match compilation_kind {
            CompilationKind::Back => {
                send_front
                    .send((
                        CompilationKind::BackPlusFront(send_front.clone()),
                        SignalKind::TriggerAndCompileOther,
                    ))
                    .unwrap();
            }
            CompilationKind::Front => {
                send_front
                    .send((CompilationKind::Front, SignalKind::TriggerTwice))
                    .unwrap();
            }
            CompilationKind::BackPlusFront(_) => {
                // send_front
                //     .send((CompilationKind::BackPlusFront, SignalKind::Trigger))
                //     .unwrap();
            }
        }
    }
}

async fn watch_dir<Fu: Future<Output = ()> + 'static>(
    path: &str,
    callback: impl Fn(
            Arc<RwLock<Option<(i64, CompilationKind)>>>,
            broadcast::Sender<(CompilationKind, SignalKind)>,
            broadcast::Sender<(CompilationKind, SignalKind)>,
        ) -> Fu
        + Copy,
    send_back: broadcast::Sender<(CompilationKind, SignalKind)>,
    send_front: broadcast::Sender<(CompilationKind, SignalKind)>,
    compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>,
) {
    println!("watching {}", path);

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
                            println!(
                                "RECOMPILING, CURRENT STATE: {:?}",
                                (*compiling_state.read().await)
                            );
                            callback(
                                compiling_state.clone(),
                                send_back.clone(),
                                send_front.clone(),
                            )
                            .await;
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
