#!/usr/bin/env sh

cargo build --package=artcord --no-default-features --features=ssr &&
cargo build --package=artcord-leptos --target=wasm32-unknown-unknown --no-default-features --features=hydrate &&
rm -rf ./target/site/* &&
mkdir -p ./target/site/pkg &&
cp -r ./assets/* ./target/site
tailwindcss -i input.css -o output.css -c tailwind.config.js &&
cp ./output.css ./target/site/pkg/leptos_start5.css &&
wasm-bindgen ./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm --no-typescript --target web --out-dir ./target/site/pkg --out-name leptos_start5 &&
./target/debug/artcord
