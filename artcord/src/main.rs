use artcord_actix::server::create_server;
use dotenv::dotenv;
use std::{env, sync::Arc};
use futures::try_join;

#[actix_web::main]
async fn main() {
    dotenv().ok();

    println!("TESTING MEEEEEEEEEEE 5 3");


    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());

    let assets_root_dir = env::var("ASSETS_ROOT_DIR").unwrap_or("./target/site".to_string());
    let gallery_root_dir = env::var("GALLERY_ROOT_DIR").unwrap_or("./gallery/".to_string());

    
    let assets_root_dir = Arc::new(assets_root_dir);
    let gallery_root_dir = Arc::new(gallery_root_dir);

    let web_server = create_server(gallery_root_dir, assets_root_dir).await;

    let r = try_join!(
        async { web_server.await.or_else(|e| Err(e.to_string())) },
     //   async { bot_server.start().await.or_else(|e| Err(e.to_string())) } a
    );

    r.unwrap();

    println!("hello");
}