use futures::try_join;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::process::Stdio;
use std::{ops::Deref, path::Path, sync::Arc, time::Duration};
use tokio::{
    process::Command,
    sync::{oneshot, Mutex},
    time::sleep,
};

#[tokio::main]
async fn main() {
    let path = "artcord-builder";
   
    let one_is_canceled = Arc::new(Mutex::new(false));
    let (send_cancel, receive_cancel) = oneshot::channel::<()>();

    
    let one: tokio::task::JoinHandle<()> = tokio::spawn({
        let one_is_canceled = one_is_canceled.clone();
        async move {
            let c = Command::new("cargo").arg("build").arg("--package").arg("artcord-leptos").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
            println!("EXIT CODE: {}", c);
            let c = Command::new("rm").arg("-r").arg("./target/site").stdout(Stdio::piped()).spawn().unwrap().wait_with_output().await.unwrap().status.code().unwrap();
            println!("EXIT CODE: {}", c);
            // let c = Command::new("cargo").arg("build").arg("--package").arg("artcord-leptos").spawn().unwrap();
            // tokio::spawn(async move {
            //     let a = c.stdout;
            // });
            // c.wait_with_output();
            // let c = String::from_utf8(c).unwrap();
            // println!("{}", c);
        
            // let mut index: usize = 0;
            // loop {
            //     println!("one: {}", index);
            //     let one_is_canceled = { *one_is_canceled.lock().await.deref() };
            //     if one_is_canceled {
            //         println!("cancel sent");
                    
            //         break;
            //     }
            //     sleep(Duration::from_secs(1)).await;
            //     index += 1;
            // }

            send_cancel.send(()).unwrap();
        }
    });

    // let two: tokio::task::JoinHandle<()> = tokio::spawn(async move {
    //     let mut index: usize = 0;
    //     loop {
    //         println!("two: {}", index);
    //         sleep(Duration::from_secs(1)).await;
    //         index += 1;
    //     }
    // });

    let three: tokio::task::JoinHandle<()> = tokio::spawn({
        let one_is_canceled = one_is_canceled.clone();
        async move {
            let mut index: usize = 0;
            loop {
                println!("CANCELING ONE STARTED: {}", index);
                sleep(Duration::from_secs(5)).await;
                index += 1;
                {
                    let mut one_is_canceled = one_is_canceled.lock().await;
                    *one_is_canceled = true;
                }
                let r = receive_cancel.await;
                if let Ok(_) = r {
                    println!("ONE WAS CANCELD.");
                } else {
                    println!("reveve error {} ONE WAS CANCELD.", r.err().unwrap());
                }
                break;
            }
        }
    });

    // let bb = aa.await;

   // let front = watch_dir(path);

    let r = try_join!(
      //  async { front.await },
        async {
            one.await;
            Ok::<(), ()>(())
        },
        // async {
        //     two.await;
        //     Ok(())
        // },
        async {
            three.await;
            Ok::<(), ()>(())
        },
    );

    r.unwrap();
}

async fn watch_dir(path: &str) -> Result<(), ()> {
    println!("watching {}", path);

    let (mut tx, mut rx) = channel(1);

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

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => println!("changed: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
