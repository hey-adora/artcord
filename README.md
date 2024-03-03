# ArtCord - A website for sharing art from discord.

## Getting Started

```sh
cargo install cargo-leptos
./style/install.sh
cp .env.example .env # and fill in the values
```

Serve the website:

```sh
cargo leptos servd
```

_Note: to update the CSS simply run `./style/update.sh`._

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

Made with ❤️ by [hey\_\_adora](https://discord.com/users/1159037321283375174) (add me on Discord).
