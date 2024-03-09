// use futures::{try_join, Future, FutureExt};
// use futures::{
//     channel::mpsc::{channel, Receiver},
//     SinkExt, StreamExt,
// };
// use notify::event::{AccessKind, AccessMode};
// use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
// use tokio::sync::RwLock;
// use tokio::task::JoinHandle;
// use std::borrow::BorrowMut;
// use std::cell::RefCell;
// use std::ops::DerefMut;
// use std::process::Stdio;
// use std::rc::Rc;
// use std::{ops::Deref, path::Path, sync::Arc, time::Duration};
// use tokio::{
//     process::Command,
//     sync::{oneshot, Mutex},
//     time::sleep,
// };
// use futures::TryFutureExt;

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use chrono::Utc;
use futures::Future;
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

async fn compile_full() -> ([Command; 9], [&'static str; 9]) {
    let mut c1 = Command::new("cargo");
    c1.arg("build").arg("--package").arg("artcord-leptos");

    let mut c2 = Command::new("rm");
    c2.arg("-r").arg("./target/site");

    let mut c3 = Command::new("mkdir");
    c3.arg("./target/site");

    let mut c4 = Command::new("mkdir");
    c4.arg("./target/site/pkg");

    let mut c5 = Command::new("cp");
    c5.arg("-r").arg("./assets/.").arg("./target/site/");

    let mut c6 = Command::new("cp");
    c6.arg("./style/output.css")
        .arg("./target/site/pkg/leptos_start5.css");

    let mut c7 = Command::new("wasm-bindgen");
    c7.arg("./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm")
        .arg("--no-typescript")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg("./target/site/pkg")
        .arg("--out-name")
        .arg("leptos_start5");

    let mut c8 = Command::new("cargo");
    c8.arg("build").arg("--package").arg("artcord");

    let mut c9 = Command::new("./target/debug/artcord");

    let mut commands = [c1, c2, c3, c4, c5, c6, c7, c8, c9];
    let command_names = [
        "Built wasm",
        "Deleted folder target/site",
        "Created folder target/site",
        "Created folder target/site/pkg",
        "Coppied assets to target/site",
        "Copied style to target/site/pkg/leptos_start5.css",
        "Generated target/site/pkg/leptos_start5.js",
        "Built artcord x86",
        "Launched artcord",
    ];

    (commands, command_names)
}

async fn compile_front() -> ([Command; 7], [&'static str; 7]) {
    let mut c1 = Command::new("cargo");
    c1.arg("build").arg("--package").arg("artcord-leptos");

    let mut c2 = Command::new("rm");
    c2.arg("-r").arg("./target/site");

    let mut c3 = Command::new("mkdir");
    c3.arg("./target/site");

    let mut c4 = Command::new("mkdir");
    c4.arg("./target/site/pkg");

    let mut c5 = Command::new("cp");
    c5.arg("-r").arg("./assets/.").arg("./target/site/");

    let mut c6 = Command::new("cp");
    c6.arg("./style/output.css")
        .arg("./target/site/pkg/leptos_start5.css");

    let mut c7 = Command::new("wasm-bindgen");
    c7.arg("./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm")
        .arg("--no-typescript")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg("./target/site/pkg")
        .arg("--out-name")
        .arg("leptos_start5");

    let mut commands = [c1, c2, c3, c4, c5, c6, c7];
    let command_names = [
        "Built wasm",
        "Deleted folder target/site",
        "Created folder target/site",
        "Created folder target/site/pkg",
        "Coppied assets to target/site",
        "Copied style to target/site/pkg/leptos_start5.css",
        "Generated target/site/pkg/leptos_start5.js",
    ];

    (commands, command_names)
}

//a a a

const SMOOTHING_TOLERENCE: i64 = 100;

// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
// enum CompType {
//     Front(i64),
//     Back(i64),
//    // Assets(i64)
// }

#[tokio::main]
async fn main() {
    let path_be = "artcord";
    let path_be_actix = "artcord-actix";
    let path_be_mongodb = "artcord-mongodb";
    let path_be_serenity = "artcord-serenity";
    let path_be_state = "artcord-state";
    let path_be_tungstenite = "artcord-tungstenite";
    let path_fe = "artcord-leptos";
    let path_assets = "assets";

    let compiling_backend: Arc<RwLock<Option<i64>>> = Arc::new(RwLock::new(None));

    let (send_front, mut recv_front) = broadcast::channel::<()>(1);
    let (send_back, mut recv_back) = broadcast::channel::<()>(1);

    // let (send_start, mut recv_start) = mpsc::channel::<()>(1000);

    let front_comp = {
        let compiling_backend = compiling_backend.clone();
        async move {
            //compile_front(recv_cancel).await;
    
            let (mut commands, command_names) = compile_front().await;
            loop {
                recv_front.recv().await.unwrap();
    
                // if (*compiling_backend.read().await).is_some_and(|time| Utc::now().timestamp_millis() - time <= SMOOTHING_TOLERENCE) {
                //     continue; a
                // }
    
                if (*compiling_backend.read().await).is_some() {
                    println!("FE: SKIPPING.");
                    continue;
                }
    
                for (i, command) in commands.iter_mut().enumerate() {
                    let mut command = command.spawn().unwrap();
    
                    let recv = recv_front.recv();
    
    
                    select! {
                        c = command.wait() => {
                            let success = c.and_then(|v| Ok(v.success())).unwrap_or(false);
                            if !success {
                                println!("FE Error: {}", command_names[i]);
                                break;
                            } else {
                                println!("FE Finished: {}", command_names[i]);
                            }
                        },
                        _ = recv => {
                            command.kill().await.unwrap();
                            println!("FE Killed: {}", command_names[i]);
                            break;
                        }
                    };
                }
            }
        }
    };

    let back_comp = async move {
        //compile_front(recv_cancel).awaita aaa a a 
        let (mut commands, command_names) = compile_full().await;
        let len = commands.len();
        loop {
            recv_back.recv().await.unwrap();
            {
                let mut compiling_backend = compiling_backend.write().await;
                *compiling_backend = Some(Utc::now().timestamp_millis());
            }
            for (i, command) in commands.iter_mut().enumerate() {
                if i.checked_sub(1).unwrap_or(0) == len {
                    let mut compiling_backend = compiling_backend.write().await;
                    *compiling_backend = None;
                }
                let mut command = command.spawn().unwrap();
                let recv = recv_back.recv();

                select! {
                    c = command.wait() => {
                        
                        let success = c.and_then(|v| Ok(v.success())).unwrap_or(false);
                        if !success {
                            println!("BE Error: {}", command_names[i]);
                            break;
                        } else {
                            println!("BE Finished: {}", command_names[i]);
                        }
                    },
                    _ = recv => {
                        command.kill().await.unwrap();
                        println!("BE Killed: {}", command_names[i]);
                        break;
                    }
                };
            }
        }
    };

    //let a = Utc::now().timestamp_millis();
    
    // send_front.send(()).unwrap(); 333
    send_back.send(()).unwrap();
    

    let back_comp = tokio::spawn(back_comp);
    let front_comp = tokio::spawn(front_comp);

    let watch_fe = watch_dir(path_fe, send_front.clone());
    let watch_assets = watch_dir(path_assets, send_front);

    let watch_be = watch_dir(path_be, send_back.clone());
    let watch_be_actix = watch_dir(path_be_actix, send_back.clone());
    let watch_be_mongodb = watch_dir(path_be_mongodb, send_back.clone());
    let watch_be_serenity = watch_dir(path_be_serenity, send_back.clone());
    let watch_be_state = watch_dir(path_be_state, send_back.clone());
    let watch_be_tungstenite = watch_dir(path_be_tungstenite, send_back);

    let r = join!(
        watch_fe,
        watch_assets,
        back_comp, 
        watch_be, 
        watch_be_actix, 
        watch_be_mongodb, 
        watch_be_serenity, 
        watch_be_state, 
        watch_be_tungstenite
    );
}

// async fn watch_dir<Fu: Future<Output = ()> + 'static>(path: &str, callback: impl Fn() -> Fu + Copy)
async fn watch_dir(path: &str, send_signal: broadcast::Sender<()>) {
    println!("watching {}", path);

    // let callback_wrap = RefCell::new(|| callback);
    // let aaa = || async {};

    let (mut tx, mut rx) = mpsc::channel(1000);

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
                            send_signal.send(()).unwrap();
                            //callback().await;
                            // let callback_wrap = &*callback_wrap.borrow();
                            // let a = callback_wrap();
                            //.await;
                            
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    //Ok::<(), ()>(())
}
