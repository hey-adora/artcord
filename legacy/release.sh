#!/usr/bin/env sh

# #cargo build --package=artcord --no-default-features --features=ssr
# cargo build --package=artcord --lib --target=wasm32-unknown-unknown --no-default-features --features=hydrate
# rm -r ./target/site
# mkdir ./target/site
# mkdir ./target/site/pkg
# cp -r ./assets/* ./target/site/
# cp ./style/output.css ./target/site/pkg/leptos_start5.css
# wasm-bindgen ./target/wasm32-unknown-unknown/debug/artcord.wasm --no-typescript --target web --out-dir ./target/site/pkg --out-name leptos_start5
# #./target/debug/artcord

# cargo build --package=artcord-leptos --target=wasm32-unknown-unknown --features=csr
# rm -r ./target/site
# mkdir ./target/site
# mkdir ./target/site/pkg
# cp -r ./assets/* ./target/site/
# cp ./style/output.css ./target/site/pkg/leptos_start5.css
# wasm-bindgen ./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm --no-typescript --target web --out-dir ./target/site/pkg --out-name leptos_start5

cargo build --package=artcord --no-default-features --release
cargo build --package=artcord-leptos --target=wasm32-unknown-unknown --no-default-features --features=hydrate --profile wasm-release
rm -r ./target/site
mkdir ./target/site
mkdir ./target/site/pkg
cp -r ./assets/* ./target/site/
cp ./style/output.css ./target/site/pkg/leptos_start5.css
wasm-bindgen ./target/wasm32-unknown-unknown/wasm-release/artcord_leptos.wasm --no-typescript --target web --out-dir ./target/site/pkg --out-name leptos_start5
#wasm-snip --snip-rust-panicking-code --snip-rust-fmt-code ./target/site/pkg/leptos_start5_bg.wasm -o ./target/site/pkg/leptos_start5_bg.wasm
wasm-opt -Oz  ./target/site/pkg/leptos_start5_bg.wasm -o ./target/site/pkg/leptos_start5_bg.wasm