use leptos::prelude::*;
use server_fn::codec::Rkyv;

use app::App;
use tracing::trace;

pub mod app;
pub mod logger;
pub mod toolbox;

pub fn shell(options: LeptosOptions) -> impl IntoView {

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />

                <HydrationScripts options />
                <meta name="color-scheme" content="dark light" />
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico" />
                <link rel="stylesheet" id="leptos" href="/pkg/artbounty_1.css" />
            </head>
            <body class="bg-gray-950">
                <App />
            </body>
        </html>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    logger::simple_logger_init();
    leptos::mount::hydrate_body(App);
}
