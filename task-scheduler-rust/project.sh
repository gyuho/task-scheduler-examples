#!/bin/bash -e

cargo new task-scheduler-rust --vcs none
cd ./task-scheduler-rust

cargo install cargo-edit

# create "lib.rs" to link modules
touch ./src/lib.rs

# "id.rs"
touch ./src/id.rs
cargo add rand

# "notify.rs"
touch ./src/notify.rs
cargo add uuid --features v4
cargo add tokio --features full

# "echo.rs"
touch ./src/echo.rs
cargo add serde --features derive
cargo add serde_json

# "apply.rs"
touch ./src/apply.rs
# none

# "server.rs"
touch ./src/server.rs
cargo add http
cargo add futures
cargo add hyper --features full

# "main.rs"
cargo add clap@3.0.0-beta.2
