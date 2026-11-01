#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo +stable build --locked --bins
(
    unset CARGO_TARGET_DIR
    sh vendor/hibana-wash/scripts/build.sh
)
