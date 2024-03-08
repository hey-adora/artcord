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


use notify::event::{AccessKind, AccessMode};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::process::Child;
use tokio::process::Command;
use tokio::join;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio::time::timeout;
use tokio::time::Duration;
use tokio::sync::oneshot;
use tokio::sync::broadcast;

async fn compile(mut recv: broadcast::Receiver::<()>) -> () {
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
    c6.arg("./style/output.css").arg("./target/site/pkg/leptos_start5.css");

    let mut c7 = Command::new("wasm-bindgen");
    c7.arg("./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm").arg("--no-typescript").arg("--target").arg("web").arg("--out-dir").arg("./target/site/pkg").arg("--out-name").arg("leptos_start5");

    let mut c8 = Command::new("cargo");
    c8.arg("build").arg("--package").arg("artcord");

    let mut c9 = Command::new("./target/debug/artcord");

    let mut commands = [c1,c2,c3,c4,c5,c6,c7,c8,c9,];


    loop {
        for command in commands.iter_mut() {
            let mut command = command.spawn().unwrap();
            let recv = recv.recv();
            select! {
                _ = command.wait() => {
                    println!("Finished");
                },
                _ = recv => {
                    command.kill().await.unwrap();
                    println!("Killed");
                    break;
                }
            };
        }
    }
   
}


// async fn compile() -> tokio::process::Child {
//     let c = Command::new("cargo").arg("build").arg("--package").arg("artcord-leptos").status().await.unwrap();
//     println!("EXIT 1 CODE: {}", c);
//     let c = Command::new("rm").arg("-r").arg("./target/site").status().await.unwrap();
//     println!("EXIT 2 CODE: {}", c);
//     let c = Command::new("mkdir").arg("./target/site").status().await.unwrap();
//     println!("EXIT 3 CODE: {}", c);
//     let c = Command::new("mkdir").arg("./target/site/pkg").status().await.unwrap();
//     println!("EXIT 4 CODE: {}", c);
//     let c = Command::new("cp").arg("-r").arg("./assets/.").arg("./target/site/").status().await.unwrap();
//     println!("EXIT 5 CODE: {}", c);
//     let c = Command::new("cp").arg("./style/output.css").arg("./target/site/pkg/leptos_start5.css").status().await.unwrap();
//     println!("EXIT 6 CODE: {}", c);
//     let c = Command::new("wasm-bindgen").arg("./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm").arg("--no-typescript").arg("--target").arg("web").arg("--out-dir").arg("./target/site/pkg").arg("--out-name").arg("leptos_start5").status().await.unwrap();
//     println!("EXIT 7 CODE: {}", c);
//     let c = Command::new("cargo").arg("build").arg("--package").arg("artcord").status().await.unwrap();
//     println!("EXIT 8 CODE: {}", c);
//     let c: tokio::process::Child = Command::new("./target/debug/artcord").spawn().unwrap();
//     //println!("EXIT 9 CODE: {}", c);
//     c
// }



#[tokio::main]
async fn main() {
  

    // let b = async move {
    //     sleep(Duration::from_secs(5)).await;
    //     tx.send(()).unwrap();
    // };

    // join!(a, b);

    
    // rx;

    // let (send, recv) = oneshot::channel::<()>();
    // let mut command: Child = Command::new("sh").spawn().unwrap();

    // select! {
    //     _ = command.wait() => {
    //         println!("Finished");
    //     },
    //     _ = recv => {
    //         command.kill().await.unwrap();
    //         println!("Killed");
    //     }
    // }

    // let run_stuff = async {
    //     timeout(Duration::from_secs(5), async {
    //         command.wait().await.unwrap();
    //     })
    // };

    // let cancel_run = async {
    //     sleep(Duration::from_secs(5)).await;
    //     command.kill();
    // };

//    / join!(run_stuff, cancel_run);
   
//     let holder: Arc<Mutex<Option<tokio::process::Child>>> = Arc::new(Mutex::new(None));
   
//     // let one_is_canceled = Arc::new(Mutex::new(false));
//     // let (send_cancel, receive_cancel) = oneshot::channel::<()>();
//    // println!("AAAAAAAAAAAAAAAAAAA");
    
//     // let one: tokio::task::JoinHandle<()> = tokio::spawn({
//     // //    let one_is_canceled = one_is_canceled.clone();
//     //     async move {
//     //         // let c = Command::new("cargo").arg("build").arg("--package").arg("artcord-leptos").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
//     //         // println!("EXIT 1 CODE: {}", c);
//     //         // let c = Command::new("rm").arg("-r").arg("./target/site").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
//     //         // println!("EXIT 2 CODE: {}", c);
//     //         // let c = Command::new("mkdir").arg("./target/site").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
//     //         // println!("EXIT 3 CODE: {}", c);
//     //         // let c = Command::new("mkdir").arg("./target/site/pkg").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
//     //         // println!("EXIT 4 CODE: {}", c);
//     //         // let c = Command::new("cp").arg("-r").arg("./assets/.").arg("./target/site/").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
//     //         // println!("EXIT 5 CODE: {}", c);
//     //         // let c = Command::new("cp").arg("./style/output.css").arg("./target/site/pkg/leptos_start5.css").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
//     //         // println!("EXIT 6 CODE: {}", c);
//     //         // let c = Command::new("wasm-bindgen").arg("./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm").arg("--no-typescript").arg("--target").arg("web").arg("--out-dir").arg("./target/site/pkg").arg("--out-name").arg("leptos_start5").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
//     //         // println!("EXIT 7 CODE: {}", c);
//     //         // let c = Command::new("cargo").arg("build").arg("--package").arg("artcord").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
//     //         // println!("EXIT 8 CODE: {}", c);
//     //         // let c = Command::new("./target/debug/artcord").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
//     //         // println!("EXIT 9 CODE: {}", c);

//     //         let c = Command::new("cargo").arg("build").arg("--package").arg("artcord-leptos").status().await.unwrap();
//     //         println!("EXIT 1 CODE: {}", c);
//     //         let c = Command::new("rm").arg("-r").arg("./target/site").status().await.unwrap();
//     //         println!("EXIT 2 CODE: {}", c);
//     //         let c = Command::new("mkdir").arg("./target/site").status().await.unwrap();
//     //         println!("EXIT 3 CODE: {}", c);
//     //         let c = Command::new("mkdir").arg("./target/site/pkg").status().await.unwrap();
//     //         println!("EXIT 4 CODE: {}", c);
//     //         let c = Command::new("cp").arg("-r").arg("./assets/.").arg("./target/site/").status().await.unwrap();
//     //         println!("EXIT 5 CODE: {}", c);
//     //         let c = Command::new("cp").arg("./style/output.css").arg("./target/site/pkg/leptos_start5.css").status().await.unwrap();
//     //         println!("EXIT 6 CODE: {}", c);
//     //         let c = Command::new("wasm-bindgen").arg("./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm").arg("--no-typescript").arg("--target").arg("web").arg("--out-dir").arg("./target/site/pkg").arg("--out-name").arg("leptos_start5").status().await.unwrap();
//     //         println!("EXIT 7 CODE: {}", c);
//     //         let c = Command::new("cargo").arg("build").arg("--package").arg("artcord").status().await.unwrap();
//     //         println!("EXIT 8 CODE: {}", c);
//     //         let c = Command::new("./target/debug/artcord").status().await.unwrap();
//     //         println!("EXIT 9 CODE: {}", c);
//     //         // let c = Command::new("./target/debug/artcord").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
//     //         // println!("EXIT 9 CODE: {}", c);
//     //         // let c = Command::new("cargo").arg("build").arg("--package").arg("artcord-leptos").spawn().unwrap();
//     //         // tokio::spawn(async move {
//     //         //     let a = c.stdout;
//     //         // });
//     //         // c.wait_with_output();
//     //         // let c = String::from_utf8(c).unwrap();
//     //         // println!("{}", c);
        
//     //         // let mut index: usize = 0;
//     //         // loop {
//     //         //     println!("one: {}", index);
//     //         //     let one_is_canceled = { *one_is_canceled.lock().await.deref() };
//     //         //     if one_is_canceled {
//     //         //         println!("cancel sent");
                    
//     //         //         break;
//     //         //     }
//     //         //     sleep(Duration::from_secs(1)).await;
//     //         //     index += 1;
//     //         // }

//     //         //send_cancel.send(()).unwrap();
//     //     }
//     // });

//    // let one = Arc::new(one);

//     // let two: tokio::task::JoinHandle<()> = tokio::spawn(
//     //     async move {

//     //         sleep(Duration::from_secs(5)).await;
//     //         one.abort();
//     //         // let mut index: usize = 0;
//     //         // loop {
//     //         //     println!("two: {}", index);
//     //         //     sleep(Duration::from_secs(1)).await;
//     //         //     index += 1;
//     //         // }
//     //     }
//     // );

//     // let three: tokio::task::JoinHandle<()> = tokio::spawn({
//     //     let one_is_canceled = one_is_canceled.clone();
//     //     async move {
//     //         let mut index: usize = 0;
//     //         loop {
//     //             println!("CANCELING ONE STARTED: {}", index);
//     //             sleep(Duration::from_secs(5)).await;
//     //             index += 1;
//     //             {
//     //                 let mut one_is_canceled = one_is_canceled.lock().await;
//     //                 *one_is_canceled = true;
//     //             }
//     //             let r = receive_cancel.await;
//     //             if let Ok(_) = r {
//     //                 println!("ONE WAS CANCELD.");
//     //             } else {
//     //                 println!("reveve error {} ONE WAS CANCELD.", r.err().unwrap());
//     //             }
//     //             break;
//     //         }
//     //     }
//     // });

//     // let bb = aa.await; a
 
//     // let front = watch_dir(path); a a a a a a a a
    

    
//     let mut c: tokio::process::Child = Command::new("./target/debug/artcord").spawn().unwrap();
//     let mut c = Arc::new(RefCell::new(c));
//     // c.wait_with_output().
//     // //let aaa = c.wait();
//     // let c = Arc::new(RwLock::new(c));
//     // let c2 = c.clone();

//     // let c = {
//     //     let mut c = c.write().await;
//     //     let mut c = &mut *c;
//     //     //Command::kill_on_drop(&mut self, kill_on_drop);
//     //     c.wait()
//     // };
//     //c.await;
//    // let mut c = *c;
   
//     //let c = c.read().await.wait_with_output();

//     let h: Rc<Option<std::pin::Pin<Box<dyn Future<Output = Result<std::process::ExitStatus, std::io::Error>> + Send>>>> = Arc::new(None);
//    // let h: Arc<Option<Box<dyn Future<Output = ()>>>> = Arc::new(Box::new(None));
//     let a = vec![Option::Some((async move {}))];


    

//     let a = async {
//         println!("STARTED");
//         let  c = &*c;
//         // let c = {
           


//         // };
//         let mut c = c.borrow_mut();
//         let mut c = &mut *c;
//         let a: std::pin::Pin<Box<dyn Future<Output = Result<std::process::ExitStatus, std::io::Error>> + Send>> = c.wait().boxed();
//         let a = &*a;
//         //let a = a.into_future();
//         //c.await.unwrap();
//         // let cc = c.wait();
//         // tokio::spawn(async {
//         //     c.kill().await;
//         // });
//         // cc.await.unwrap();
//         println!("ENDED");
//         Ok::<(), ()>(())
//     };

//     let b = async {
//         sleep(Duration::from_secs(5)).await;
//         println!("KILLING");
//         let c = &*c;
//         let c = &mut *c.borrow_mut();
//         c.kill().await.unwrap();
//         println!("KILLED");
//         // let cc = c.wait();
//         // tokio::spawn(async {
//         //     c.kill().await;
//         // });
//         // cc.await.unwrap();
//        // println!("ENDED");
//         Ok::<(), ()>(())
//     };


//     try_join!(
//         a,
//         b
//         // async {
//         //     sleep(Duration::from_secs(5)).await;
//         //     println!("KILLING");
//         //     c.kill().await;
//         //     println!("KILLED");
//         //     Ok::<(), ()>(())
//         // } a 
//     ); a a a a a


    let path = "artcord-builder";

    let (send_cancel, mut recv_cancel) = broadcast::channel::<()>(2);
    let mut a = async move {
        compile(recv_cancel).await;
    };

    let h = tokio::spawn(a);

    let r = join!(
       h,
       async { 
            println!("watching {}", path);

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

            watcher.watch(path.as_ref(), RecursiveMode::Recursive).unwrap();



            while let Some(res) = rx.recv().await {
                match res {
                    Ok(event) => {
                        if let EventKind::Access(kind) = event.kind {
                            if let AccessKind::Close(kind) = kind {
                                if let AccessMode::Write = kind {
                                    
                                    println!("RECOMPILING");
                                    send_cancel.send(()).unwrap();
                                    // let mut holder = holder.lock().await;
                                    // if let Some(e) = &mut *holder {
                                    //     e.kill().await.unwrap();
                                    //     //try_join!(a);
                                    // }
                                    // // let t = tokio::spawn(async move {
                                    // //     ;
                                    // // });
                                    // let t = compile().await;
                                    // *holder = Some(t);
                                }
                            }
                        }
                    },
                    Err(e) => println!("watch error: {:?}", e),
                }
            };

            Ok::<(), ()>(())
        },

    );
       
//     //     // async {
//     //     //     one.await;
//     //     //     Ok::<(), ()>(())
//     //     // },
//     //     // async {
//     //     //     two.await;
//     //     //     Ok::<(), ()>(())
//     //     // },
//     //     // async { aa  aa
//     //     //     three.await;
//     //     //     Ok::<(), ()>(())
//     //     // }, a a a 
//     // );

//     //r.unwrap();
}



// async fn watch_dir(path: &str) -> Result<(), ()> {
//     println!("watching {}", path);

   

   

//     Ok(())
// }
