FROM archlinux:latest

RUN pacman -Sy
RUN pacman --noconfirm -S rustup sudo git base-devel go
RUN useradd -mG wheel crab
RUN echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" >> "/etc/sudoers"
USER crab
WORKDIR /home/crab
RUN git clone https://aur.archlinux.org/yay
WORKDIR /home/crab/yay
RUN makepkg --noconfirm -si PKGBUILD
RUN yay --noconfirm -S nvm
RUN bash -c "source /usr/share/nvm/init-nvm.sh && nvm install 21.6.1 && npm -g i tailwindcss"
RUN rustup toolchain install stable
RUN rustup target add wasm32-unknown-unknown
RUN rustup default stable
RUN cargo install cargo-leptos
RUN cargo install wasm-bindgen-cli