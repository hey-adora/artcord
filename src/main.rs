use std::sync::LazyLock;

use axum::Router;
use heyadora_art::{app::App, shell};
use leptos::{logging, prelude::*};
use leptos_axum::{generate_route_list, LeptosRoutes};
use surrealdb::{engine::remote::ws, Surreal};
use tower_http::compression::CompressionLayer;
use tracing::{info, trace, trace_span};

static DB: LazyLock<Surreal<ws::Client>> = LazyLock::new(Surreal::init);

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() {
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

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let comppression_layer = CompressionLayer::new().zstd(true).gzip(true).deflate(true);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .layer(comppression_layer);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
