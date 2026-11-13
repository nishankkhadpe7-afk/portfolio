#!/usr/bin/env bash
# Build NishankOS on Vercel (or any bare Linux box): install Rust + the wasm
# target + a prebuilt Trunk binary, then produce ./dist.
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi
# shellcheck disable=SC1091
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown

TRUNK_VERSION="${TRUNK_VERSION:-v0.22.1}"
if ! command -v trunk >/dev/null 2>&1; then
  curl -sSL "https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" \
    | tar -xz -C "$HOME/.cargo/bin"
fi

trunk build --release
