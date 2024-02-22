#!/usr/bin/env sh

#cargo build --package=artcord --no-default-features --features=ssr
cargo build --package=artcord --lib --target=wasm32-unknown-unknown --no-default-features --features=hydrate
rm -r ./target/site
mkdir ./target/site
mkdir ./target/site/pkg
cp -r ./assets/* ./target/site/
cp ./style/output.css ./target/site/pkg/leptos_start5.css
wasm-bindgen ./target/wasm32-unknown-unknown/debug/artcord.wasm --no-typescript --target web --out-dir ./target/site/pkg --out-name leptos_start5
#./target/debug/artcord