#![feature(future_join)]
#![feature(box_patterns)]

use artcord::bot::create_bot;
use artcord::database::create_database;
use artcord::server::create_server;
use dotenv::dotenv;
use futures::try_join;
use std::env;

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let token = env::var("DISCORD_BOT_TOKEN").expect("ENV MISSING: DISCORD_BOT_TOKEN");
    let mongo_url = std::env::var("MONGO_URL").expect("ENV MISSING: MONGO_URL");
    let discord_default_guild =
        std::env::var("DISCORD_DEFAULT_GUILD").expect("ENV MISSING: DISCORD_DEFAULT_GUILD");

    let db = create_database(mongo_url).await;
    db.allowed_guild_insert_default(discord_default_guild)
        .await
        .unwrap();

    let mut bot_server = create_bot(db.clone(), token).await;
    let web_server = create_server(db).await;

    let r = try_join!(
        async { web_server.await.or_else(|e| Err(e.to_string())) },
        async { bot_server.start().await.or_else(|e| Err(e.to_string())) }
    );
    r.unwrap();

    Ok(())
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use artcord::app::*;
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(move || {
        // note: for testing it may be preferrable to replace this with a
        // more specific component, although leptos_router should still work
        view! { <App/> }
    });
}
