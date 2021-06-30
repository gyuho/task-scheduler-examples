use std::{
    io,
    io::{Error, ErrorKind},
    string::String,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::{
    select,
    sync::mpsc,
    task::{self, JoinHandle},
    time::timeout,
};

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

pub struct Applier {
    request_timeout: Duration,

    request_id_generator: id::Generator,
    notifier: Arc<notify::Notifier>,

    request_tx: mpsc::UnboundedSender<(u64, Request)>,
    stop_tx: mpsc::Sender<()>,
}

impl Applier {
    pub fn new(req_timeout: Duration) -> (Self, JoinHandle<io::Result<()>>) {
        let member_id = rand::random::<u64>();

        let (request_ch_tx, request_ch_rx) = mpsc::unbounded_channel();
        let (stop_ch_tx, stop_ch_rx) = mpsc::channel(1);

        let notifier = Arc::new(notify::Notifier::new());
        let notifier_clone = notifier.clone();

        let echo_manager = echo::Manager::new();

        let handle = task::spawn(async move {
            apply_async(notifier_clone, request_ch_rx, stop_ch_rx, echo_manager).await
        });

        (
            Self {
                request_timeout: req_timeout,

                request_id_generator: id::Generator::new(member_id, Instant::now().elapsed()),
                notifier,

                request_tx: request_ch_tx,
                stop_tx: stop_ch_tx,
            },
            handle,
        )
    }

    pub async fn stop(&self) -> io::Result<()> {
        println!("stopping applier");

        match self.stop_tx.send(()).await {
            Ok(()) => println!("sent stop_tx"),
            Err(e) => {
                println!("failed to send stop_tx: {}", e);
                return Err(Error::new(ErrorKind::Other, format!("failed to send")));
            }
        }

        println!("stopped applier");
        Ok(())
    }

    pub async fn apply(&self, req: Request) -> io::Result<String> {
        let req_id = self.request_id_generator.next();
        let resp_rx = self.notifier.register(req_id)?;

        match self.request_tx.send((req_id, req)) {
            Ok(_) => println!("scheduled a request"),
            Err(e) => {
                self.notifier
                    .trigger(req_id, format!("failed to schedule {} {}", req_id, e))?;
            }
        }

        let msg = timeout(self.request_timeout, resp_rx)
            .await
            .map_err(|_| Error::new(ErrorKind::Other, "timeout"))?;

        let rs = msg.map_err(|e| Error::new(ErrorKind::Other, format!("failed to apply {}", e)))?;
        Ok(rs)
    }
}

pub async fn apply_async(
    notifier: Arc<notify::Notifier>,
    mut request_rx: mpsc::UnboundedReceiver<(u64, Request)>,
    mut stop_rx: mpsc::Receiver<()>,
    mut echo_manager: echo::Manager,
) -> io::Result<()> {
    println!("running apply routine");
    'outer: loop {
        // select either
        // receive either from "request_rx" or "stop_rx"
        let (req_id, req) = select! {
            Some(v) = request_rx.recv() => v,
            _ = stop_rx.recv() => {
                println!("received stop_rx");
                break 'outer;
            }
        };

        match req.echo_request {
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
