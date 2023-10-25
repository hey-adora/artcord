#![feature(future_join)]

use actix::{
    Actor, ActorContext, ActorFutureExt, AsyncContext, ContextFutureSpawner, Handler, Message,
    Recipient, StreamHandler, WrapFuture,
};
use actix_files::Files;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws::{self, ProtocolError};
use futures::{future, select, try_join, TryFutureExt};
use leptos::get_configuration;
use leptos_actix::{generate_route_list, LeptosRoutes};
use rand::Rng;

use async_std::task;
use dotenv::dotenv;
use futures::future::join_all;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandGroup, CommandResult, GroupOptions, StandardFramework};
use serenity::prelude::*;

use artcord::bot::create_bot;
use artcord::server::create_server;
use artcord::database::create_database;
use std::env;
use std::future::join;
use rkyv::{Archive, Deserialize, Serialize};
use wasm_bindgen::__rt::Start;




#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db = create_database().await;
    //println!("{:#?}", db);
    let mut bot_server = create_bot(db.clone()).await;
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
