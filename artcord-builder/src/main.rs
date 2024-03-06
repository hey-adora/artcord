use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use futures::try_join;

#[tokio::main]
async fn main() {
    let path = "artcord-builder"; 
    let front = watch_dir(path);


    let r = try_join!(
        async { front.await },

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
    ).unwrap();

    watcher.watch(path.as_ref(), RecursiveMode::Recursive).unwrap();

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => println!("changed: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}