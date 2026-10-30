#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo +stable build --locked --bins
sh vendor/hibana-wash/scripts/build.sh
