use std::{
    collections::HashMap,
    io,
    io::{Error, ErrorKind},
    string::String,
};

use async_std::sync::RwLock;
use crossbeam_channel::{self, unbounded};

#[derive(Debug)]
pub struct Notifier {
    requests: RwLock<HashMap<u64, crossbeam_channel::Sender<String>>>,
}

impl Notifier {
    pub fn new() -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, id: u64) -> io::Result<crossbeam_channel::Receiver<String>> {
        println!("registering {}", id);
        let mut _mu;
        match self.requests.try_write() {
            Some(guard) => _mu = guard,
            None => {
                println!("failed to acquire lock");
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "failed to acquire lock",
                ));
            }
        }

        let (request_tx, request_rx) = unbounded();
        if _mu.get(&id).is_none() {
            _mu.insert(id, request_tx);
        } else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("duplicate request ID {}", id),
            ));
        }

        println!("registered {}", id);
        Ok(request_rx)
    }

    pub fn trigger(&self, id: u64, x: String) -> io::Result<()> {
        println!("triggering {}", id);
        let mut _mu;
        match self.requests.try_write() {
            Some(guard) => _mu = guard,
            None => {
                println!("failed to acquire lock");
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "failed to acquire lock",
                ));
            }
        }

        let request_tx;
        match _mu.get(&id) {
            Some(ch) => request_tx = ch,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("unknown request ID {}", id),
                ))
            }
        }

        match request_tx.send(x) {
            Ok(_) => {
                println!("triggered {}", &id);
                _mu.remove(&id);
            }
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("failed to send {} {}", id, e),
                ))
            }
        }

        Ok(())
    }
}

#[test]
fn test_wait() {
    let notifier = Notifier::new();

    let ret = notifier.register(100);
    assert!(ret.is_ok());
    let rx = ret.unwrap();

    use uuid::Uuid;
    let uuid = Uuid::new_v4().to_hyphenated().to_string();

    let tr = notifier.trigger(100, uuid.clone());
    assert!(tr.is_ok());

    use crossbeam_channel::{after, select};
    use std::time::Duration;
    let timeout = after(Duration::from_secs(1));
    select! {
        recv(rx) -> msg => assert_eq!(msg, Ok(uuid.clone())),
        recv(timeout) -> _ => panic!("timed out"),
    }

    let tr = notifier.trigger(100, uuid);
    assert!(!tr.is_ok());
}
