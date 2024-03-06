# ArtCord

#### This project is a art sharing website with many features to come.

> Currently under heavy refactoring/cleanup. Contributions are welcome, my discord handle `hey__adora`

Roadmap:

- [x] Homepage
- [x] Pull art from discord server.
- [x] Display art in website gallery.
- [x] Add author profiles with their art only.
- [-] Refactoring and making the code easier to work with and read. [link](https://github.com/hey-adora/artcord/issues/1)
- [ ] Add connection limit and throttle and auto block.
- [ ] Add admin dashboard for seeing ip's and connection count with ability to block.
- [ ] Add authentication by email or discord.
- [ ] Add galley sorting and filtering.
- [ ] Add fav button.
- [ ] Add user profile with user settings and their favorited art.
- [ ] Add comments.

## Build
### Build using docker.
- `docker compose up`

### Build manually.
1. `rustup toolchain install stable` - make sure you have Rust stable
2. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
3. `cargo leptos serve` - install `cargo-generate` binary (should be installed automatically in future)
4. `npm install -g sass` - install `dart-sass` (should be optional in future)

## Updating css

1. `npm -g i tailwindcss`
1. `tailwindcss -i input.css -o style/output.css -c tailwind.config.js -w`
