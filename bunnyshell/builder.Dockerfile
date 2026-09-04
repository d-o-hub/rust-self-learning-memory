# Builder image for Bunnyshell persistent remote compilation.
# Tracks the stable toolchain declared in rust-toolchain.toml and pre-installs
# everything CI needs (clippy, rustfmt, llvm-tools, nextest, mold) so that
# `scripts/bns-cargo.sh` can run `cargo build --workspace` via `bns exec`
# without paying the apt/cargo-install cost on every environment recreate.
FROM rust:1-slim-bookworm

ENV CARGO_TERM_COLOR=always \
  RUST_BACKTRACE=1 \
  CARGO_TARGET_DIR=/app/target \
  CARGO_INCREMENTAL=1 \
  CARGO_NET_RETRIES=5

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    clang \
    mold \
    curl \
    git \
    jq \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/*

RUN rustup toolchain install stable \
    --component rustfmt,clippy,llvm-tools-preview \
    --target x86_64-unknown-linux-gnu \
  && rustup default stable

RUN cargo install --locked cargo-nextest

# TEMPORARY-ROOT-FIXUP (reverted in next commit): run as root once so we can
# chown the cargo volumes (populated by root during initial verification) to
# the builder user. Restored to USER builder immediately after.
# RUN useradd --uid 1000 --create-home --shell /bin/bash builder \
#   && chown -R builder:builder /app /usr/local/cargo
# USER builder

WORKDIR /app

CMD ["sleep", "infinity"]
