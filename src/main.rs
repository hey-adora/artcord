//use crate::app::*;
use axum::{Router, routing::{get}};
use heyadora_art::{shell, App};
use leptos::{logging, prelude::*};
use leptos_axum::{generate_route_list, LeptosRoutes};
use tracing::{trace, trace_span, info};
//use server_fns_axum::*;

// cargo make cli: error: unneeded `return` statement
#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() {
    // simple_logger::init_with_level(log::Level::Error)
    //     .expect("couldn't initialize logging");

    tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .unwrap();

    trace!("started!");

    // Setting this to None means we'll be using cargo-leptos and its env vars
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    //let addr = "0.0.0.0:3000";

    // build our application with a route a
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        ;
      

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

  