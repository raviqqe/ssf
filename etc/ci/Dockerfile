FROM archlinux/base

RUN pacman -Syu --noconfirm llvm npm rustup
RUN rustup default stable
RUN cargo install cargo-audit
RUN npm install -g cspell
