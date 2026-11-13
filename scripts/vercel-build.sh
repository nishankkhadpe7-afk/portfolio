#!/usr/bin/env bash
# Build NishankOS on Vercel (or any bare Linux box): make sure rustup + the
# wasm32 target + a prebuilt Trunk binary exist, then produce ./dist.
set -euo pipefail

export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export PATH="$CARGO_HOME/bin:$PATH"

# Vercel's image can ship a bare `cargo` without rustup, so test for rustup
# itself — it's what installs the wasm32 target.
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | RUSTUP_INIT_SKIP_PATH_CHECK=yes sh -s -- -y --profile minimal --default-toolchain none --no-modify-path
fi
# rust-toolchain.toml pins the channel and target; this installs them.
rustup toolchain install stable --profile minimal --target wasm32-unknown-unknown

TRUNK_VERSION="${TRUNK_VERSION:-v0.22.1}"
if ! command -v trunk >/dev/null 2>&1; then
  # musl build: statically linked, so it runs on Vercel's older glibc.
  mkdir -p "$CARGO_HOME/bin"
  curl -sSL "https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-musl.tar.gz" \
    | tar -xz -C "$CARGO_HOME/bin"
fi

rustc --version
trunk --version
trunk build --release
