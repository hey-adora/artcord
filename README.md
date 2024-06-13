# ArtCord

## Description
A fully working website for posting/sharing art. 

> Currently under heavy development. Contributions are welcome, my discord handle `hey__adora`

Roadmap:

- [x] Homepage
- [x] Pull art from discord server.
- [x] Display art in website gallery.
- [x] Add author profiles with their art only.
- [x] Refactoring and making the code easier to work with and read. [link](https://github.com/hey-adora/artcord/issues/1)
- [ ] (in progress) Add connection limit and throttle and auto block.
- [ ] Add admin dashboard for seeing ip's and connection count with ability to block.
- [ ] Add authentication by email or discord.
- [ ] Add galley sorting and filtering.
- [ ] Add fav button.
- [ ] Add user profile with user settings and their favorited art.
- [ ] Add comments.

## Build

### Build using docker.

- `docker compose up`

### Build manually on Arch linux.

before you can run it manually, you need to setup the mongodb database first. Easiest way would be `cd mongo && docker compose up` which will only start the database instance.

And then you can run the website with:

1. `sudo pacman -S rustup sudo git base-devel go binaryen nodejs npm`
2. `npm -g i tailwindcss`
3. `rustup toolchain install stable`
4. `rustup target add wasm32-unknown-unknown`
5. `rustup default stable`
6. `cargo install wasm-bindgen-cli`
7. `cargo run --package artcord-builder`

it should also work with `cargo-leptos serve` but i dont use it personally.