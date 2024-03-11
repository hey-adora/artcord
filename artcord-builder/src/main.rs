use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::ffi::{OsStr, OsString};
use std::path;
use std::pin::Pin;
use std::process::ExitStatus;
use std::rc::Rc;
use std::sync::Arc;

use chrono::Utc;
use futures::future::join_all;
use futures::{Future, FutureExt};
use notify::event::{AccessKind, AccessMode};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::join;
use tokio::process::Child;
use tokio::process::Command;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::RwLock;
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio::time::timeout;
use tokio::time::Duration;



const SMOOTHING_TOLERENCE: i64 = 100;

#[derive(Clone, Debug)]
enum SignalKind {
    Trigger,
    TriggerTwice,
    TriggerAndCompileOther(broadcast::Sender<(CompilationKind, SignalKind)>)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CompilationKind {
    Front,
    Back,
    BackPlusFront
}

#[tokio::main]
async fn main() {
    let compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>> = Arc::new(RwLock::new(None));
    let (send_front, mut recv_front) = broadcast::channel::<(CompilationKind, SignalKind)>(1);
    let (send_back, mut recv_back) = broadcast::channel::<(CompilationKind, SignalKind)>(1);

    let mut commands_backend = build_commands([
        vec!["cargo","build","--package","artcord-leptos",],
        vec!["rm","-r","./target/site",],
        vec!["mkdir","./target/site",],
        vec!["mkdir","./target/site/pkg",],
        vec!["cp","-r","./assets/.","./target/site/",],
        vec!["cp","./style/output.css","./target/site/pkg/leptos_start5.css",],
        vec!["wasm-bindgen","./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm","--no-typescript","--target","web","--out-dir","./target/site/pkg","--out-name","leptos_start5",],
        vec!["cargo","build","--package","artcord",],
        vec!["./target/debug/artcord",],
        ]).await;

    let mut commands_frontend = build_commands([
        vec!["cargo","build","--package","artcord-leptos",],
        vec!["rm","-r","./target/site",],
        vec!["mkdir","./target/site",],
        vec!["mkdir","./target/site/pkg",],
        vec!["cp","-r","./assets/.","./target/site/"],
        vec!["cp","./style/output.css","./target/site/pkg/leptos_start5.css",],
        vec!["wasm-bindgen","./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm","--no-typescript","--target","web","--out-dir","./target/site/pkg","--out-name","leptos_start5",],
        ]).await;

    let mut futs: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();

    let paths_backend = ["artcord", "artcord-actix", "artcord-mongodb", "artcord-serenity", "artcord-state", "artcord-tungstenite"];
        let paths_frontend = ["artcord-leptos", "assets"];

        for path in paths_backend {
            let fut = watch_dir(path, watch_dir_back_callback, send_back.clone(), send_front.clone(), compiling_state.clone());
            futs.push(fut.boxed());
        }

        for path in paths_frontend {
            let fut = watch_dir(path, watch_dir_front_callback, send_back.clone(), send_front.clone(), compiling_state.clone());
            futs.push(fut.boxed());
        }

    let handler_frontend = proccess((send_front.clone(), recv_front), compiling_state.clone(), false, "FE", &mut commands_frontend).boxed();
    futs.push(handler_frontend);

    let handler_backend = proccess((send_back.clone(), recv_back), compiling_state.clone(), false, "BE", &mut commands_backend).boxed();
    futs.push(handler_backend);

    join_all(futs).await;
}

async fn build_commands<I>(commands_parts: I) -> Vec<(Command, String)> where
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
                },
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

async fn proccess(channel: (broadcast::Sender<(CompilationKind, SignalKind)>, broadcast::Receiver<(CompilationKind, SignalKind)>), compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>, inifite: bool, name: &str, commands: &mut [(Command, String)]) {
     let (send, mut recv) = channel;
     let command_count = commands.len();

     loop {
         let (compilation_kind, signal_kind) = recv.recv().await.unwrap();
         ( *(compiling_state.clone().write().await) )= Some((Utc::now().timestamp_millis(), compilation_kind));
         if let CompilationKind::BackPlusFront = compilation_kind  {
             continue;
         }

         'command_loop: for (i, ( command, command_name)) in commands.iter_mut().enumerate() {
             let mut command = command.spawn().unwrap();

             select! {
                command_return = command.wait() => {
                    let good = proccess_on_finish(i, command_return, command, command_name, inifite, command_count, compiling_state.clone(), name).await;

                    if !good {
                        break;
                    }
                 },
                 received_value = recv.recv() => {
                    proccess_on_trigger(i, received_value, command, send.clone(), command_name, name).await;
                    break 'command_loop;
                 }
             };
         }
     }
}

async fn proccess_on_trigger(i: usize, received_signal: Result<(CompilationKind, SignalKind), broadcast::error::RecvError>, mut command: Child, send: broadcast::Sender<(CompilationKind, SignalKind)>, command_name: &str, name: &str) {
    println!("{} Killed: {}", name, command_name);

    command.kill().await.unwrap();

    let Ok((compilation_kind, signal_kind)) = received_signal else {
        println!("{} recv error: {}", name, received_signal.err().and_then(|e|Some(e.to_string())).unwrap_or_else(|| "uwknown error".to_string()));
        return;
    };
    
    match signal_kind {
        SignalKind::TriggerAndCompileOther(send) => {
            send.send((compilation_kind, SignalKind::TriggerTwice)).unwrap();
        }
        SignalKind::TriggerTwice => {
            send.send((compilation_kind, SignalKind::Trigger)).unwrap();
        }
        SignalKind::Trigger => {
            
        }
    }
}

async fn proccess_on_finish(i: usize, command_return: Result<ExitStatus, std::io::Error>, mut command: Child, command_name: &str, infinite: bool, command_count: usize, compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>, name: &str) -> bool {
    if i == command_count.checked_sub(if infinite { 2 } else { 1 }).unwrap_or(0) {
        (*compiling_state.write().await) = None;
    }
   

    let Ok(command_return) = command_return else {
        println!("{} recv error: {}", name, command_return.err().and_then(|e|Some(e.to_string())).unwrap_or_else(|| "uwknown error".to_string()));
        return false;
    };
    
    let good = command_return.success();
        if good {
            println!("{} Finished: {}", name, command_name);
            true
        } else {
            println!("{} Error[{}]: {}, ", name, command_return.code().unwrap_or(0), command_name);
            false
        }
}

async fn watch_dir_back_callback(compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>, send_back: broadcast::Sender<(CompilationKind, SignalKind)>, send_front: broadcast::Sender<(CompilationKind, SignalKind)>) {
    let compiling_state_ref = &*compiling_state.read().await;
    let Some(compiling_state_copy) = compiling_state_ref else {
        send_back.send((CompilationKind::Back, SignalKind::Trigger));
        return;
    };

    let current_time = Utc::now().timestamp_millis();
    let (past_time, compilation_kind) = compiling_state_copy;

    let time_passed = current_time - past_time;
    if time_passed > SMOOTHING_TOLERENCE {
        match compilation_kind {
            CompilationKind::Back => {
                send_back.send((CompilationKind::Back, SignalKind::TriggerTwice));
            },
            CompilationKind::Front => {
                send_front.send((CompilationKind::Back,SignalKind::TriggerAndCompileOther(send_back.clone())));
            },
            CompilationKind::BackPlusFront => {
                send_back.send((CompilationKind::Back, SignalKind::TriggerTwice));
            }
        }
    }
}

async fn watch_dir_front_callback(compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>, send_back: broadcast::Sender<(CompilationKind, SignalKind)>, send_front: broadcast::Sender<(CompilationKind, SignalKind)>) {
    let compiling_state_ref = &*compiling_state.read().await;
    let Some(compiling_state_copy) = compiling_state_ref else {
        send_front.send((CompilationKind::Front, SignalKind::Trigger));
        return;
    };

    let current_time = Utc::now().timestamp_millis();
    let (past_time, compilation_kind) = compiling_state_copy;

    let time_passed = current_time - past_time;
    if time_passed > SMOOTHING_TOLERENCE {
        match compilation_kind {
            CompilationKind::Back => {
                send_front.send((CompilationKind::BackPlusFront, SignalKind::Trigger));
            },
            CompilationKind::Front => {
                send_front.send((CompilationKind::Front, SignalKind::TriggerTwice));
            },
            CompilationKind::BackPlusFront => {
                send_front.send((CompilationKind::BackPlusFront, SignalKind::Trigger));
            }
        }
    }
}


async fn watch_dir<Fu: Future<Output = ()> + 'static>(path: &str, callback: impl Fn(Arc<RwLock<Option<(i64, CompilationKind)>>>, broadcast::Sender<(CompilationKind, SignalKind)>, broadcast::Sender<(CompilationKind, SignalKind)>) -> Fu + Copy, send_back: broadcast::Sender<(CompilationKind, SignalKind)>, send_front: broadcast::Sender<(CompilationKind, SignalKind)>, compiling_state: Arc<RwLock<Option<(i64, CompilationKind)>>>) {
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
                            println!("RECOMPILING");
                            callback(compiling_state.clone(), send_back.clone(), send_front.clone()).await;
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
