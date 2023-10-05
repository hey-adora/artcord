cd vendors/cargo-leptos
rustup toolchain install nightly
rustup default nightly
cargo build --release

cd ../trunk
cargo build --release

cd ../tailwindcss
rustup toolchain install stable
rustup default stable
npm i
npm run build

rustup default nightly