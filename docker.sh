#!/bin/bash

source /usr/share/nvm/init-nvm.sh
PATH=$PATH:~/.cargo/bin/
cargo run --package artcord-builder &
code-server /app
