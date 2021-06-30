#!/usr/bin/env bash
set -xue

# cargo clean

cargo build --release

# cargo install cargo-bloat
# Get a list of the biggest dependencies in the release build
cargo bloat --crates --release

ls -lah ./target/release
