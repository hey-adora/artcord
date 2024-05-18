FROM archlinux:latest

RUN pacman -Sy
RUN pacman --noconfirm -S rustup sudo git base-devel go binaryen
RUN useradd -mG wheel crab
RUN echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" >> "/etc/sudoers"
USER crab
WORKDIR /home/crab
RUN git clone https://aur.archlinux.org/yay
WORKDIR /home/crab/yay
RUN makepkg --noconfirm -si PKGBUILD
RUN yay --noconfirm -S nvm code-server
RUN bash -c "source /usr/share/nvm/init-nvm.sh && nvm install 21.6.1 && npm -g i tailwindcss"
RUN rustup toolchain install stable
RUN rustup target add wasm32-unknown-unknown
RUN rustup default stable
RUN cargo install cargo-leptos wasm-bindgen-cli
RUN mkdir -p /home/crab/.config/code-server
RUN printf "bind-addr: 0.0.0.0:8080\nauth: none\npassword: be7e07638dd24555f63eff9d\ncert: false\n" > /home/crab/.config/code-server/config.yaml
RUN code-server --install-extension jeff-hykin.better-cpp-syntax
RUN code-server --install-extension vadimcn.vscode-lldb
RUN code-server --install-extension serayuzgur.crates
RUN code-server --install-extension rust-lang.rust-analyzer
RUN code-server --install-extension hbenl.vscode-test-explorer
RUN code-server --install-extension ms-azuretools.vscode-docker
RUN printf "{\"workbench.colorTheme\":\"Default Dark Modern\",\"rust-analyzer.check.command\":\"clippy\",\"files.autoSave\":\"off\"}" > /home/crab/.local/share/code-server/User/settings.json
RUN printf "[{\"key\":\"ctrl+s\",\"command\":\"workbench.action.files.saveFiles\"},{\"key\":\"ctrl+s\",\"command\":\"-workbench.action.files.save\"}]" > /home/crab/.local/share/code-server/User/keybindings.json
ENTRYPOINT [ "/app/docker.sh" ]