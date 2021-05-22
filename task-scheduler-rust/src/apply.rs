use std::{
    io,
    io::{Error, ErrorKind},
    string::String,
    time::{Duration, Instant},
};

use async_std::{sync::Arc, task};
use crossbeam_channel::{self, after, select, unbounded};

use crate::echo;
use crate::id;
use crate::notify;

#[derive(Debug)]
pub struct Request {
    pub echo_request: Option<echo::Request>,
}

impl Request {
    pub fn new() -> Self {
        Self { echo_request: None }
    }
}

#[derive(Clone, Debug)]
pub struct Applier {
    request_timeout: Duration,

    request_id_generator: Arc<id::Generator>,
    notifier: Arc<notify::Notifier>,

    request_tx: crossbeam_channel::Sender<(u64, Request)>,
    request_rx: crossbeam_channel::Receiver<(u64, Request)>,

    stop_tx: crossbeam_channel::Sender<()>,
    stop_rx: crossbeam_channel::Receiver<()>,

    done_tx: crossbeam_channel::Sender<()>,
    done_rx: crossbeam_channel::Receiver<()>,

    echo_manager: Arc<echo::Manager>,
}

impl Applier {
    pub fn new(req_timeout: Duration) -> Self {
        let member_id = rand::random::<u64>();

        let (request_ch_tx, request_ch_rx) = unbounded();
        let (stop_ch_tx, stop_ch_rx) = unbounded();
        let (done_ch_tx, done_ch_rx) = unbounded();

        Self {
            request_timeout: req_timeout,

            request_id_generator: Arc::new(id::Generator::new(member_id, Instant::now().elapsed())),
            notifier: Arc::new(notify::Notifier::new()),

            request_tx: request_ch_tx,
            request_rx: request_ch_rx,
            stop_tx: stop_ch_tx,
            stop_rx: stop_ch_rx,
            done_tx: done_ch_tx,
            done_rx: done_ch_rx,

            echo_manager: Arc::new(echo::Manager::new()),
        }
    }

    pub async fn start(&self) -> io::Result<()> {
        task::spawn(apply_async(
            self.notifier.clone(),
            self.request_rx.clone(),
            self.stop_rx.clone(),
            self.done_tx.clone(),
            self.echo_manager.clone(),
        ));
        // async runtimes are cooperative, picking a thread and may compute forever
        // so, multiple workers may block up the whole runtime
        // manually call "yield" or "yield_now" to allow newly-spawned task to execute first
        // ref. https://docs.rs/tokio/1.6.0/tokio/task/fn.yield_now.html
        task::yield_now().await;

        Ok(())
    }

    pub async fn stop(&self) -> io::Result<()> {
        println!("stopping applier");
        match self.stop_tx.send(()) {
            Ok(_) => println!("sent stop_tx"),
            Err(e) => {
                println!("failed to send stop_tx {}", e);
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("failed to send {}", e),
                ));
            }
        }
        match self.done_rx.recv() {
            Ok(_) => println!("received done_rx"),
            Err(e) => {
                println!("failed to receive done_rx {}", e);
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("failed to recv {}", e),
                ));
            }
        };

        println!("stopped applier");
        Ok(())
    }

    pub async fn apply(&self, req: Request) -> io::Result<String> {
        let req_id = self.request_id_generator.next();

        let resp_rx;
        match self.notifier.register(req_id) {
            Ok(ch) => resp_rx = ch,
            Err(e) => return Err(e),
        }

        match self.request_tx.send((req_id, req)) {
            Ok(_) => println!("scheduled a request"),
            Err(e) => {
                match self
                    .notifier
                    .trigger(req_id, format!("failed to schedule {} {}", req_id, e))
                {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        let timeout = after(self.request_timeout);
        select! {
            recv(resp_rx) -> msg => match msg {
                Ok(rs) => Ok(rs),
                Err(e) => Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to apply {}", e),
                    ))
            },
            recv(timeout) -> _ => return Err(Error::new(ErrorKind::Other, "timeout"))
        }
    }
}

pub async fn apply_async(
    notifier: Arc<notify::Notifier>,
    request_rx: crossbeam_channel::Receiver<(u64, Request)>,
    stop_rx: crossbeam_channel::Receiver<()>,
    done_tx: crossbeam_channel::Sender<()>,
    echo_manager: Arc<echo::Manager>,
) -> io::Result<()> {
    println!("running apply routine");
    'outer: loop {
        // select either
        // receive either from "stop_rx" or "request_rx"
        let req_id;
        let req;
        select! {
            recv(request_rx) -> pair => {
                match pair {
                    Ok(v) => {
                        req_id = v.0;
                        req = v.1;
                    },
                    Err(e) => {
                        println!("failed to receive request {}", e);
                        continue 'outer;
                    }
                }
            }
            recv(stop_rx) -> _ => {
                println!("received stop_rx");
                match done_tx.send(()) {
                    Ok(_) => println!("sent done_tx"),
                    Err(e) => panic!("failed to sent done_tx {}", e),
                }
                break 'outer;
            }
        }

        match &req.echo_request {
            Some(v) => match echo_manager.apply(&v) {
                Ok(rs) => match notifier.trigger(req_id, rs) {
                    Ok(_) => {}
                    Err(e) => println!("failed to trigger {}", e),
                },
                Err(e) => {
                    println!("failed to apply {}", e);
                    match notifier.trigger(req_id, format!("failed {}", e)) {
                        Ok(_) => {}
                        Err(e) => println!("failed to trigger {}", e),
                    }
                }
            },
            None => {}
        }
    }

    Ok(())
}
