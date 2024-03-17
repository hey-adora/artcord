use artcord_actix::server::create_server;
use artcord_tungstenite::create_websockets;
use cfg_if::cfg_if;
use dotenv::dotenv;
use futures::try_join;
use std::{env, sync::Arc};
use tracing::{info};

#[actix_web::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init().unwrap();
    //tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::builder().parse_lossy("artcord=trace")).try_init().unwrap();
    // cfg_if! {
    //     if #[cfg(feature = "production")] {
    //         tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init().unwrap();
    //     } else {
    //         tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init().unwrap();
    //     }
    // }

    info!("started!");

    let path = env::current_dir().unwrap();
    info!("current working directory is {}", path.display());

    let assets_root_dir = env::var("ASSETS_ROOT_DIR").unwrap_or("./target/site".to_string());
    info!("current assets directory is {}", assets_root_dir);
    let gallery_root_dir = env::var("GALLERY_ROOT_DIR").unwrap_or("./gallery/".to_string());
    info!("current gallery directory is {}", gallery_root_dir);

    let assets_root_dir = Arc::new(assets_root_dir);
    let gallery_root_dir = Arc::new(gallery_root_dir);

    let web_server = create_server(gallery_root_dir, assets_root_dir).await;
    let web_sockets = create_websockets();

    let r = try_join!(
        async { web_server.await.or_else(|e| Err(e.to_string())) },
        async { web_sockets.await.or_else(|e| Err(e.to_string())) },
        //   async { bot_server.start().await.or_else(|e| Err(e.to_string())) } a a aa a a a
    );

    r.unwrap();
}
