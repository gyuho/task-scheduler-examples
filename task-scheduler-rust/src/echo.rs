use std::{
    io,
    io::{Error, ErrorKind},
};

use async_std::{sync::Arc, sync::RwLock};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Request {
    pub kind: String,
    pub message: Option<String>,
}

impl Request {
    pub fn new() -> Self {
        Self {
            kind: String::from(""),
            message: None,
        }
    }
}

pub fn parse_request(b: &[u8]) -> io::Result<Request> {
    let v: Request = match serde_json::from_slice(b) {
        Ok(val) => val,
        Err(e) => {
            println!("failed to serde_json::from_slice {}", e);
            return Err(Error::new(ErrorKind::InvalidInput, "invalid JSON"));
        }
    };
    Ok(v)
}

#[test]
fn test_parse_request() {
    let ret = parse_request("{\"kind\":\"create\",\"message\":\"hello\"}".as_bytes());
    assert!(ret.is_ok());
    let t = ret.unwrap();
    assert!(t.kind.to_owned() == "create");
    assert!(to_string(t.message.to_owned()) == "hello");
}

#[derive(Debug)]
pub struct Manager {
    mu: Arc<RwLock<()>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            mu: Arc::new(RwLock::new(())),
        }
    }

    pub fn apply(&self, req: &Request) -> io::Result<String> {
        println!("applying echo request");
        let _mu;
        match self.mu.try_write() {
            Some(guard) => _mu = guard,
            None => {
                println!("failed to acquire lock");
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "failed to acquire lock",
                ));
            }
        }

        // check every 5-second
        // use std::{thread, time::Duration};
        // thread::sleep(Duration::from_secs(5));

        match req.kind.as_str() {
            "create" => Ok(format!(
                "SUCCESS create {}",
                to_string(req.message.to_owned())
            )),
            "delete" => Ok(format!(
                "SUCCESS delete {}",
                to_string(req.message.to_owned())
            )),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("unexpected none field value for kind {}", req.kind),
            )),
        }
    }
}

fn to_string(o: Option<String>) -> String {
    match o {
        Some(v) => v,
        None => String::new(),
    }
}
