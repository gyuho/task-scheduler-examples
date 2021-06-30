#!/bin/bash -e

cargo new task-scheduler-rust --vcs none
cd ./task-scheduler-rust

cargo install cargo-edit

cd ./src
touch lib.rs

# "id.rs"
touch id.rs
cargo add rand

# "notify.rs"
touch notify.rs
cargo add uuid --features v4
cargo add tokio --features full

# "echo.rs"
touch echo.rs
cargo add serde --features derive
cargo add serde_json

# "apply.rs"
touch apply.rs
# none

# "server.rs"
touch server.rs
cargo add http
cargo add futures
cargo add hyper --features full

# "main.rs"
cargo add clap@3.0.0-beta.2
