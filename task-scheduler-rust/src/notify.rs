use std::{
    collections::{hash_map::Entry, HashMap},
    io,
    io::{Error, ErrorKind},
    string::String,
    sync::{Arc, Mutex},
};

use tokio::sync::oneshot;

#[derive(Debug)]
pub struct Notifier {
    requests: Arc<Mutex<HashMap<u64, oneshot::Sender<String>>>>,
}

impl Notifier {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&self, id: u64) -> io::Result<oneshot::Receiver<String>> {
        println!("registering {}", id);

        let (request_tx, request_rx) = oneshot::channel();

        let mut locked_map = self
            .requests
            .lock()
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "failed to acquire lock"))?;

        match locked_map.entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(request_tx);
            }
            Entry::Occupied(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("duplicate request ID {}", id),
                ))
            }
        };

        println!("registered {}", id);
        Ok(request_rx)
    }

    pub fn trigger(&self, id: u64, x: String) -> io::Result<()> {
        println!("triggering {}", id);

        let mut locked_map = self
            .requests
            .lock()
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "failed to acquire lock"))?;

        match locked_map.entry(id) {
            Entry::Occupied(entry) => {
                println!("triggered {}", &id);
                let request_tx = entry.remove();
                request_tx.send(x).map_err(|e| {
                    Error::new(ErrorKind::Other, format!("failed to send {} {}", id, e))
                })?;
            }
            Entry::Vacant(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("unknown request ID {}", id),
                ))
            }
        }

        Ok(())
    }
}

#[tokio::test]
async fn test_wait() {
    use std::time::Duration;
    use tokio::time::timeout;
    use uuid::Uuid;

    let notifier = Notifier::new();

    let ret = notifier.register(100);
    assert!(ret.is_ok());
    let rx = ret.unwrap();

    let uuid = Uuid::new_v4().to_hyphenated().to_string();

    let tr = notifier.trigger(100, uuid.clone());
    assert!(tr.is_ok());

    let msg = timeout(Duration::from_secs(1), rx).await;
    assert_eq!(msg, Ok(Ok(uuid.clone())));

    let tr = notifier.trigger(100, uuid);
    assert!(!tr.is_ok());
}
