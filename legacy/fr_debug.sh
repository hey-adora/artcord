#!/usr/bin/env sh

# cargo build --package=artcord --no-default-features --features=ssr,serve_csr
cargo build --package=artcord-leptos --target=wasm32-unknown-unknown --no-default-features --features=csr
rm -rf ./target/site &&
mkdir ./target/site &&
mkdir ./target/site/pkg &&
cp -r ./assets/* ./target/site
tailwindcss -i input.css -o output.css -c tailwind.config.js &&
cp ./output.css ./target/site/pkg/leptos_start5.css &&
cp ./artcord-http/index.html ./target/site/index.html &&
wasm-bindgen ./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm --no-typescript --target web --out-dir ./target/site/pkg --out-name leptos_start5
# ./target/debug/artcord
