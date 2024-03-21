use artcord_actix::server::create_server;
use artcord_mongodb::database::DB;
use artcord_tungstenite::create_websockets;
use cfg_if::cfg_if;
use dotenv::dotenv;
use futures::try_join;
use std::{env, sync::Arc};
use tracing::info;
use tracing::trace;





#[actix_web::main]
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
    //         tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init().unwrap();
    //     } else {
    //         tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init().unwrap();
    //     }
    // } 


    trace!("started!");

    

    let path = env::current_dir().unwrap();
    trace!("current working directory is {}", path.display());

    let assets_root_dir = env::var("ASSETS_ROOT_DIR").unwrap_or("./target/site".to_string());
    trace!("current assets directory is {}", assets_root_dir);
    let gallery_root_dir = env::var("GALLERY_ROOT_DIR").unwrap_or("./gallery/".to_string());
    trace!("current gallery directory is {}", gallery_root_dir);
    let mongodb_url = env::var("MONGO_URL").unwrap_or("mongodb://root:U2L63zXot4n5@localhost:27017".to_string());
    trace!("current gallery directory is {}", gallery_root_dir);

    let assets_root_dir = Arc::new(assets_root_dir);
    let gallery_root_dir = Arc::new(gallery_root_dir);
    let db = DB::new(mongodb_url).await;

    let web_server = create_server(gallery_root_dir, assets_root_dir).await;
    let web_sockets = create_websockets();

    let r = try_join!(
        async { web_server.await.or_else(|e| Err(e.to_string())) },
        async { web_sockets.await.or_else(|e| Err(e.to_string())) },
        //   async { bot_server.start().await.or_else(|e| Err(e.to_string())) } a a aa a a a
    );

    r.unwrap();
}
