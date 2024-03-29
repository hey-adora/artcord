use artcord_actix::server::create_server;
use artcord_mongodb::database::DB;
use artcord_serenity::create_bot::create_bot;
use artcord_tungstenite::create_websockets;
use cfg_if::cfg_if;
use dotenv::dotenv;
use futures::try_join;
use std::{env, sync::Arc};
use tracing::error;
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

    let assets_root_dir = env::var("ASSETS_ROOT_DIR").unwrap_or("./target/site".to_string());
    let gallery_root_dir = env::var("GALLERY_ROOT_DIR").unwrap_or("./gallery/".to_string());
    let mongodb_url =
        env::var("MONGO_URL").unwrap_or("mongodb://root:U2L63zXot4n5@localhost:27017".to_string());
    let discord_bot_token = env::var("DISCORD_BOT_TOKEN").ok();
    let discord_bot_default_guild = env::var("DISCORD_DEFAULT_GUILD").ok();

    trace!("current working directory is {}", path.display());
    trace!("current assets directory is {}", assets_root_dir);
    trace!("current gallery directory is {}", gallery_root_dir);
    trace!("current gallery directory is {}", gallery_root_dir);

    //let assets_root_dir = Arc::new(assets_root_dir);
    //let gallery_root_dir = Arc::new(gallery_root_dir);
    let db = DB::new(mongodb_url).await;
    let db = Arc::new(db);

    let web_server = create_server(&gallery_root_dir, &assets_root_dir).await;
    let web_sockets = create_websockets(db.clone());

    if let Some(discord_bot_token) = discord_bot_token {
        let mut discord_bot = create_bot(db.clone(), &discord_bot_token, &gallery_root_dir, discord_bot_default_guild).await;

        let r = try_join!(
            async { web_server.await.or_else(|e| Err(e.to_string())) },
            async { web_sockets.await.or_else(|e| Err(e.to_string())) },
            async {
                discord_bot.start().await;
                Ok(())
            },
        );
        r.unwrap();
        
    } else {
        error!("DISCORD_BOT_TOKEN in .env is missing, bot will not start.");
        let r = try_join!(
            async { web_server.await.or_else(|e| Err(e.to_string())) },
            async { web_sockets.await.or_else(|e| Err(e.to_string())) },
        );

        r.unwrap();
    }
}
