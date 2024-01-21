#![feature(future_join)]
#![feature(box_patterns)]
#![allow(unused_variables, unused_imports)]

use artcord::bot::create_bot;

use artcord::bot::create_bot::create_bot;
use artcord::database::create_database::create_database;
use artcord::server::create_server::{create_server, TOKEN_SIZE};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use dotenv::dotenv;
use futures::try_join;
use jsonwebtoken::encode;
use rand::Rng;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::Write;
use std::sync::Arc;

pub fn get_env_bytes<F: Fn() -> String>(
    key: &str,
    base64: bool,
    default_val: Option<F>,
) -> Vec<u8> {
    let secret = std::env::var(key);
    if let Ok(secret) = secret {
        if base64 {
            BASE64_STANDARD.decode(secret).expect(&format!(
                "{} in .env file is invalid, must be encoded in base64.",
                key
            ))
        } else {
            secret.as_bytes().to_vec()
        }
    } else {
        let Some(default_val) = default_val else {
            panic!("{} in .env file was not provided.", key);
        };

        let val = default_val();

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(".env")
            .expect("Failed to open .evn file.");

        println!("ENV: GENERATED {}.", key);

        if base64 {
            let val = BASE64_STANDARD.encode(&val);
            writeln!(file, "\n{}={}", key, val).expect("Failed to write to .evn file.");
        } else {
            writeln!(file, "\n{}={}", key, &val).expect("Failed to write to .evn file.");
        }

        val.as_bytes().to_vec()
    }
}

pub fn get_env<F: Fn() -> String>(key: &str, base64: bool, default_val: Option<F>) -> String {
    let secret = std::env::var(key);
    if let Ok(secret) = secret {
        if base64 {
            let secret = BASE64_STANDARD.decode(secret).expect(&format!(
                "{} in .env file is invalid, must be encoded in base64.",
                key
            ));
            String::from_utf8(secret).expect(&format!(
                "{} in .env file is invalid, must be UTF8 compatible.",
                key
            ))
        } else {
            secret
        }
    } else {
        let Some(default_val) = default_val else {
            panic!("{} in .env file was not provided.", key);
        };

        let val = default_val();

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(".env")
            .expect("Failed to open .evn file.");

        println!("ENV: GENERATED {}.", key);

        if base64 {
            let val = BASE64_STANDARD.encode(&val);
            writeln!(file, "\n{}={}", key, val).expect("Failed to write to .evn file.");
        } else {
            writeln!(file, "\n{}={}", key, &val).expect("Failed to write to .evn file.");
        }

        val
    }
}

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let assets_root_dir = env::var("ASSETS_ROOT_DIR").expect("ENV MISSING: ASSETS_ROOT_DIR");
    let gallery_root_dir = env::var("GALLERY_ROOT_DIR").expect("ENV MISSING: GALLERY_ROOT_DIR");
    let token = env::var("DISCORD_BOT_TOKEN").expect("ENV MISSING: DISCORD_BOT_TOKEN");
    let mongo_url = std::env::var("MONGO_URL").expect("ENV MISSING: MONGO_URL");
    let discord_default_guild =
        std::env::var("DISCORD_DEFAULT_GUILD").expect("ENV MISSING: DISCORD_DEFAULT_GUILD");
    let pepper_base64 = std::env::var("PEPPER_BASE64").expect("ENV MISSING: PEPPER_BASE64");
    //std::env::
    let jwt_secret: Vec<u8> = get_env_bytes(
        "JWT_SECRET_BASE64",
        true,
        Some(|| {
            BASE64_STANDARD.encode(
                (0..TOKEN_SIZE)
                    .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
                    .collect::<String>(),
            )
        }),
    );
    let jwt_secret: Arc<Vec<u8>> = Arc::new(jwt_secret);
    // let jwt_secret_base64: String = {
    //     let secret = std::env::var("JWT_SECRET_BASE64");
    //     if let Ok(secret) = secret {
    //         let secret = BASE64_STANDARD.decode(secret);
    //         if let Ok(secret) = secret {
    //             let secret = String::from_utf8(secret);
    //             if let Ok(secret) = secret {
    //                 secret
    //             } else {
    //                 panic!("JWT_SECRET_BASE64 in .env file is invalid, must be UTF8 compatible.");
    //             }
    //         } else {
    //             panic!("JWT_SECRET_BASE64 in .env file is invalid, must be encoded in base64.");
    //         }
    //     } else {
    //         let token: String = (0..TOKEN_SIZE)
    //             .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
    //             .collect();
    //         let token_bse64 = BASE64_STANDARD.encode(&token);
    //         println!("GENERATED: {}", &token_bse64);
    //         let file = File::open("./.env");
    //
    //         std::env::set_var("JWT_SECRET_BASE64", token_bse64);
    //
    //         token
    //     }
    //     // let token: String = (0..TOKEN_SIZE)
    //     //     .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
    //     //     .collect();
    // };

    let assets_root_dir: Arc<String> = Arc::new(assets_root_dir);
    let gallery_root_dir: Arc<String> = Arc::new(gallery_root_dir);

    let pepper = BASE64_STANDARD
        .decode(pepper_base64)
        .expect("Failed to decode PEPPER_BASE64 from .env, invalid base64?");
    let pepper = String::from_utf8(pepper).expect("Failed to generate string from decoded PEPPER_BASE64 from .env, pepper must contain ascii characters from 32 to 126");
    let pepper = Arc::new(pepper);

    let db = Arc::new(create_database(mongo_url).await);
    db.allowed_guild_insert_default(discord_default_guild)
        .await
        .unwrap();

    let mut bot_server = create_bot(db.clone(), token, gallery_root_dir.as_str()).await;
    let web_server = create_server(db, gallery_root_dir, assets_root_dir, pepper, jwt_secret).await;

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
