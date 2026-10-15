#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo +1.95.0 build --locked --bins
RUSTFLAGS='-C link-arg=--initial-memory=65536 -C link-arg=--max-memory=65536 -C link-arg=-zstack-size=16384' \
  cargo +1.95.0 build --locked --manifest-path guests/shell/Cargo.toml --target wasm32-wasip1 --release
sh vendor/hibana-wash/scripts/build.sh
