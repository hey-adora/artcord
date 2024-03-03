# ArtCord - A website for sharing art from discord.

> Currently under heavy refactoring/cleanup. Contributions are welcome, my discord handle hey\_\_adora

## Getting Started

Install `cargo-leptos`:

```sh
cargo install cargo-leptos
cp .env.example .env # and fill in the values
```

Serve the website:

```sh
cargo leptos serve
```

## Roadmap

- [x] Homepage
- [x] Pull art from discord server.
- [x] Display art in website gallery.
- [x] Add author profiles with their art only.
- [-] [Refactoring and making the code easier to work with and read.](https://github.com/hey-adora/artcord/issues/1)
- [ ] Add connection limit and throttle and auto block.
- [ ] Add admin dashboard for seeing ip's and connection count with ability to block.
- [ ] Add authentication by email or discord.
- [ ] Add galley sorting and filtering.
- [ ] Add fav button.
- [ ] Add user profile with user settings and their favorited art.
- [ ] Add comments.
