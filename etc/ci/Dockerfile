FROM archlinux/base

RUN pacman -Syu --noconfirm base-devel llvm npm rustup
RUN rustup default stable
RUN cargo install cargo-audit cargo-release
RUN npm install -g cspell
