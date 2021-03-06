#![warn(clippy::all)]

use std::time::Duration;

use clap::{crate_version, Clap};

mod server;

#[derive(Clap)]
#[clap(version = "0.2.0")]
struct Opts {
    #[clap(long, default_value = "3000")]
    port: u16,
    #[clap(long, default_value = "10")]
    request_timeout_seconds: u64,
}

fn main() {
    let opts: Opts = Opts::parse();
    println!("crate version {}", crate_version!(),);

    let srv = server::Handler::new(opts.port, Duration::from_secs(opts.request_timeout_seconds));

    let future_task = srv.start();
    let rt = tokio::runtime::Runtime::new().unwrap();

    // we can run tokio within async_std and the other way around
    // NOTE: DO NOT CALL THIS RECURSIVELY
    // e.g. "Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) attempted to block the current thread while the thread is being used to drive asynchronous tasks"
    let ret = rt.block_on(future_task);
    match ret {
        Ok(_) => println!("server done"),
        Err(e) => println!("server aborted {}", e),
    };
}
